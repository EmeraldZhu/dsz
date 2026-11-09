//! One-click driver installs: download a pinned release from its official
//! GitHub page, verify it, and run its installer (Windows asks for admin).
//!
//! - usbip-win2 0.9.8.1, the current release: checked against the SHA-256
//!   GitHub publishes for it. Pinned rather than the newest: each release
//!   has changed the attach request (`virt::usbip::PLUGIN_LAYOUTS`), so a
//!   newer one may not attach until the app knows it. Not 0.9.7.8, which
//!   can crash Windows (see `virt::usbip::blocked`). 0.9.7.7 and 0.9.8.0
//!   attach when they are already installed.
//! - HidHide 1.5.230: its release publishes no hash, so the installer is
//!   pinned by the SHA-256 of the official asset *and* must carry a valid
//!   Authenticode signature from its author (both have to pass). The
//!   signing chain is checked for revocation when Windows can reach the
//!   revocation servers.
//!
//! Each install downloads into its own new folder under `%TEMP%`, and the
//! installer is held open with read-only sharing from the moment it is
//! hashed until the elevated process has started. Another program of the
//! same user can't replace it between the check and the admin prompt.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use parking_lot::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Package {
    Usbip,
    HidHide,
}

enum Check {
    Sha256([u8; 32]),
    /// Both the pinned hash and a valid signature from this signer.
    Sha256AndSigner([u8; 32], &'static str),
}

struct Release {
    name: &'static str,
    url: &'static str,
    file: &'static str,
    check: Check,
}

const fn nibble(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        _ => panic!("hex"),
    }
}

const fn hex32(s: &str) -> [u8; 32] {
    let b = s.as_bytes();
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = nibble(b[2 * i]) << 4 | nibble(b[2 * i + 1]);
        i += 1;
    }
    out
}

impl Package {
    fn release(self) -> Release {
        match self {
            Package::Usbip => Release {
                name: "usbip-win2 0.9.8.1",
                url: "https://github.com/vadimgrn/usbip-win2/releases/download/v.0.9.8.1/USBip-0.9.8.1-x64.exe",
                file: "USBip-0.9.8.1-x64.exe",
                check: Check::Sha256(hex32(
                    "38cad6d4432b52d5bb9409d9ad03b72fdffc4ada4cd3a48fbeca1a2752a8518a",
                )),
            },
            Package::HidHide => Release {
                name: "HidHide 1.5.230",
                url: "https://github.com/nefarius/HidHide/releases/download/v1.5.230.0/HidHide_1.5.230_x64.exe",
                file: "HidHide_1.5.230_x64.exe",
                // SHA-256 of the official release asset, computed on 2026-11-06.
                check: Check::Sha256AndSigner(
                    hex32("f4bbbcb82e6258641b887c74bc81c4c5f66e4aa811808dfc304347687b7605f6"),
                    "Nefarius Software Solutions e.U.",
                ),
            },
        }
    }

    pub fn name(self) -> &'static str {
        self.release().name
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Downloading,
    Verifying,
    /// The installer is open (after the admin prompt).
    Installing,
    /// The installer closed; `ok` is its exit status.
    Finished { ok: bool },
    Failed(String),
}

fn states() -> &'static Mutex<HashMap<Package, Status>> {
    static S: OnceLock<Mutex<HashMap<Package, Status>>> = OnceLock::new();
    S.get_or_init(Default::default)
}

pub fn status(p: Package) -> Option<Status> {
    states().lock().get(&p).cloned()
}

fn set(p: Package, s: Status) {
    states().lock().insert(p, s);
}

/// Start installing `p` in the background; `wake` runs on each change so
/// the UI can repaint. Does nothing while one is already running.
pub fn start(p: Package, wake: impl Fn() + Send + 'static) {
    if matches!(
        status(p),
        Some(Status::Downloading | Status::Verifying | Status::Installing)
    ) {
        return;
    }
    set(p, Status::Downloading);
    std::thread::spawn(move || {
        let r = run(p, &wake);
        if let Err(e) = r {
            log::warn!("installing {}: {e}", p.name());
            set(p, Status::Failed(e));
        }
        wake();
    });
}

/// A new, empty folder for one install: never a fixed path another program
/// could have prepared (a planted file, junction or link).
fn fresh_dir() -> Result<PathBuf, String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let base = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    for attempt in 0..16u32 {
        let dir = base.join(format!("DSZ-install-{}-{nanos:x}-{attempt}", std::process::id()));
        // `create_dir`, not `create_dir_all`: it fails if the folder exists.
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Err("Couldn't create a folder for the download.".into())
}

/// Open `file` so that nobody else can write, rename or delete it while the
/// returned handle is alive (reading, which the loader and WinVerifyTrust
/// need, is still allowed).
fn lock_for_reading(file: &Path) -> Result<std::fs::File, String> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
    std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(file)
        .map_err(|e| e.to_string())
}

fn passes(check: &Check, data: &[u8], file: &Path) -> Result<bool, String> {
    Ok(match *check {
        Check::Sha256(want) => sha256(data)? == want,
        Check::Sha256AndSigner(want, who) => {
            sha256(data)? == want && signer(file).as_deref() == Some(who)
        }
    })
}

fn run(p: Package, wake: &dyn Fn()) -> Result<(), String> {
    let rel = p.release();
    let dir = fresh_dir()?;
    let result = install_from(p, &rel, &dir, wake);
    let _ = std::fs::remove_dir_all(&dir);
    result
}

fn install_from(p: Package, rel: &Release, dir: &Path, wake: &dyn Fn()) -> Result<(), String> {
    use std::io::Read;
    let file = dir.join(rel.file);
    log::info!("downloading {} from {}", rel.name, rel.url);
    download(rel.url, &file)?;
    set(p, Status::Verifying);
    wake();
    // Hash the bytes read through the locked handle, and keep it open until
    // the installer runs, so what was checked is what runs.
    let mut lock = lock_for_reading(&file)?;
    let mut data = Vec::new();
    lock.read_to_end(&mut data).map_err(|e| e.to_string())?;
    if !data.starts_with(b"MZ") || !passes(&rel.check, &data, &file)? {
        return Err(format!(
            "The downloaded {} didn't pass its check, so it wasn't run.",
            rel.name
        ));
    }
    drop(data);
    set(p, Status::Installing);
    wake();
    let code = run_elevated(&file);
    drop(lock);
    let code = code?;
    log::info!("{} installer exited with {code}", rel.name);
    // 0, or 3010 ("restart required").
    set(p, Status::Finished { ok: code == 0 || code == 3010 });
    Ok(())
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn wide_path(p: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    p.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
}

pub(crate) fn download(url: &str, to: &PathBuf) -> Result<(), String> {
    use windows_sys::Win32::System::Com::Urlmon::URLDownloadToFileW;
    let (u, f) = (wide(url), wide_path(to));
    let hr = unsafe { URLDownloadToFileW(std::ptr::null_mut(), u.as_ptr(), f.as_ptr(), 0, std::ptr::null_mut()) };
    if hr != 0 {
        return Err(format!(
            "Download failed (0x{:08X}). Check the internet connection and try again.",
            hr as u32
        ));
    }
    Ok(())
}

pub(crate) fn sha256(data: &[u8]) -> Result<[u8; 32], String> {
    use windows_sys::Win32::Security::Cryptography::{BCryptHash, BCRYPT_SHA256_ALG_HANDLE};
    let mut out = [0u8; 32];
    let st = unsafe {
        BCryptHash(
            BCRYPT_SHA256_ALG_HANDLE,
            std::ptr::null(),
            0,
            data.as_ptr(),
            data.len() as u32,
            out.as_mut_ptr(),
            32,
        )
    };
    if st != 0 {
        return Err(format!("SHA-256 failed (0x{:08X})", st as u32));
    }
    Ok(out)
}

/// The signature check could not learn whether a certificate was revoked
/// (no network, or the revocation server didn't answer).
const REVOCATION_UNKNOWN: [u32; 2] = [
    0x800B_010E, // CERT_E_REVOCATION_FAILURE
    0x8009_2013, // CRYPT_E_REVOCATION_OFFLINE
];

/// The signer's common name when `file` has a valid, trusted Authenticode
/// signature; `None` otherwise.
///
/// Revocation is checked for the whole chain (except the root), so a
/// revoked signing certificate fails. When revocation can't be checked
/// (offline), the signature is checked without it, as before; the
/// installer's pinned SHA-256 still has to match as well.
pub(crate) fn signer(file: &Path) -> Option<String> {
    match verify_signature(file, true) {
        Ok(name) => name,
        Err(rc) if REVOCATION_UNKNOWN.contains(&rc) => {
            log::warn!(
                "{}: certificate revocation couldn't be checked (0x{rc:08X}); checking the signature without it",
                file.display()
            );
            verify_signature(file, false).ok().flatten()
        }
        Err(_) => None,
    }
}

/// WinVerifyTrust on `file`: the signer's name when trusted, or the error.
fn verify_signature(file: &Path, revocation: bool) -> Result<Option<String>, u32> {
    use windows_sys::Win32::Security::Cryptography::{CertGetNameStringW, CERT_NAME_SIMPLE_DISPLAY_TYPE};
    use windows_sys::Win32::Security::WinTrust::*;
    let path = wide_path(file);
    let mut fi: WINTRUST_FILE_INFO = unsafe { std::mem::zeroed() };
    fi.cbStruct = std::mem::size_of::<WINTRUST_FILE_INFO>() as u32;
    fi.pcwszFilePath = path.as_ptr();
    let mut wd: WINTRUST_DATA = unsafe { std::mem::zeroed() };
    wd.cbStruct = std::mem::size_of::<WINTRUST_DATA>() as u32;
    wd.dwUIChoice = WTD_UI_NONE;
    if revocation {
        wd.fdwRevocationChecks = WTD_REVOKE_WHOLECHAIN;
        wd.dwProvFlags = WTD_REVOCATION_CHECK_CHAIN_EXCLUDE_ROOT;
    } else {
        wd.fdwRevocationChecks = WTD_REVOKE_NONE;
    }
    wd.dwUnionChoice = WTD_CHOICE_FILE;
    wd.Anonymous.pFile = &mut fi;
    wd.dwStateAction = WTD_STATEACTION_VERIFY;
    let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
    let rc = unsafe {
        WinVerifyTrust(
            std::ptr::null_mut(),
            &mut action,
            &mut wd as *mut _ as *mut std::ffi::c_void,
        )
    };
    let mut name = None;
    if rc == 0 {
        unsafe {
            let prov = WTHelperProvDataFromStateData(wd.hWVTStateData);
            if !prov.is_null() {
                let sgnr = WTHelperGetProvSignerFromChain(prov, 0, 0, 0);
                if !sgnr.is_null() {
                    let cert = WTHelperGetProvCertFromChain(sgnr, 0);
                    if !cert.is_null() && !(*cert).pCert.is_null() {
                        let mut buf = [0u16; 256];
                        let n = CertGetNameStringW(
                            (*cert).pCert,
                            CERT_NAME_SIMPLE_DISPLAY_TYPE,
                            0,
                            std::ptr::null(),
                            buf.as_mut_ptr(),
                            buf.len() as u32,
                        );
                        if n > 1 {
                            name = Some(String::from_utf16_lossy(&buf[..n as usize - 1]));
                        }
                    }
                }
            }
        }
    } else if !(revocation && REVOCATION_UNKNOWN.contains(&(rc as u32))) {
        log::warn!("{}: signature check failed (0x{:08X})", file.display(), rc as u32);
    }
    wd.dwStateAction = WTD_STATEACTION_CLOSE;
    unsafe {
        WinVerifyTrust(
            std::ptr::null_mut(),
            &mut action,
            &mut wd as *mut _ as *mut std::ffi::c_void,
        )
    };
    if rc == 0 {
        Ok(name)
    } else {
        Err(rc as u32)
    }
}

/// Run `file` as administrator and wait for it. Returns its exit code.
fn run_elevated(file: &Path) -> Result<u32, String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE};
    use windows_sys::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let (verb, f) = (wide("runas"), wide_path(file));
    let mut sei: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    sei.fMask = SEE_MASK_NOCLOSEPROCESS;
    sei.lpVerb = verb.as_ptr();
    sei.lpFile = f.as_ptr();
    sei.nShow = SW_SHOWNORMAL;
    if unsafe { ShellExecuteExW(&mut sei) } == 0 {
        return Err("The installer didn't start (the admin prompt was declined?).".into());
    }
    if sei.hProcess.is_null() {
        return Ok(0);
    }
    let mut code = 1u32;
    unsafe {
        WaitForSingleObject(sei.hProcess, INFINITE);
        GetExitCodeProcess(sei.hProcess, &mut code);
        CloseHandle(sei.hProcess);
    }
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_of_abc() {
        assert_eq!(
            sha256(b"abc").unwrap(),
            hex32("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
    }

    #[test]
    fn unsigned_file_has_no_signer() {
        let f = std::env::temp_dir().join(format!("dsz-unsigned-{}.exe", std::process::id()));
        std::fs::write(&f, b"MZ not really an exe").unwrap();
        assert_eq!(signer(&f), None);
        let _ = std::fs::remove_file(&f);
    }
}
