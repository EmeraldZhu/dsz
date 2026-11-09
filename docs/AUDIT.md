# Wireless DualSense on Windows: audit of 3 free tools, and the build plan

*Audited 2026-11-06 from source. Builds and tests were run where the sandbox allowed: DSZ's ds-proto crate 62/62 tests passed; DS-HapticLink 65 tests passed (its remaining failures are Windows-only); DualSenseClient couldn't be built (no .NET SDK/NuGet access). No hardware tests were run; see `hardware-test-plan.md`.*

## 1. How the audit ran (phases per project)
1. **Recon**: history, license, size, dependencies, CI, tests.
2. **Architecture**: trace the path real pad → BT → app → USB/IP → virtual pad → game, and the way back for haptics.
3. **Trust boundaries**: anything that runs elevated, downloads, listens on a socket, or hides devices.
4. **Hot path**: timing, buffering, packet building, bounds checks on untrusted input.
5. **Tests**: run whatever runs off-Windows.
6. **Findings**: each with severity, location and fix.

## 2. Side by side
| | **DSZ** | **DS-HapticLink** | **DualSenseClient** |
|---|---|---|---|
| Language / runtime | Rust, one exe, no runtime | Python + WebView2 (Nuitka build) | C# .NET 10 + Avalonia |
| License | MIT | MIT | GPL-3.0 |
| Maturity | 18 commits / 4 weeks, 1 author | 9 commits / 1 day, 1 author | 332 commits / 12 months, CI, docs |
| Tests | 285 (62 verified here) | 74 (65 verified here) | 1,490 (not runnable here) |
| CI | ❌ (added in the fork) | ❌ | ✅ |
| Virtual-pad USB/IP | In-process server, ephemeral port, direct IOCTL | Python asyncio server + `usbip.exe` CLI | Managed server, fixed ports, IOCTL with CLI fallback |
| Hiding the real pad | HidHide, crash-journaled | **No HidHide**: exclusive open after a device restart | HidHide, not journaled |
| BT haptics transport | Raw 3 kHz PCM in `0x32`; Opus speaker in `0x39` | 3 kHz int8 via FIR decimation; Opus speaker | Opus-encoded haptics in combined `0x36` |
| Auto per-game switching | ✅ (scans for native DS support) | ❌ | ✅ (by exe) |
| Linux | ❌ | ❌ | ✅ |
| Worst finding | Z1 installer TOCTOU (**fixed**) | H1 elevated writes into a user-writable folder | C1 update check passes on a missing hash |

## 3. Compared with more mature, similar projects
| Project | What it is | What it teaches |
|---|---|---|
| **DSX + DSX+** ([Steam](https://store.steampowered.com/app/1812620/DSX/)) | Paid and closed source; Bluetooth haptics in a beta | The feature target everyone is chasing; its UDP mod API is the de-facto standard (DSZ is compatible) |
| **DS5Dongle** ([GitHub](https://github.com/awalol/DS5Dongle)) | Pico 2 W firmware that shows up as a wired DualSense | No drivers, no background app, 1000 Hz. The bar for "seamless". Its BT audio/feature-report research is reused by DualSenseClient |
| **DualSenseY-v2** ([GitHub](https://github.com/WujekFoliarz/DualSenseY-v2)) | Long-running C++ tool | Stable mod API and emulation; still recommends USB for haptics |
| **DS4Windows** ([GitHub](https://github.com/schmaldeo/DS4Windows)) | The original remapper | Its Quick Charge idea is already in DSZ. It's a lesson in how a large user base exposes hardware edge cases |
| **usbip-win2** ([GitHub](https://github.com/vadimgrn/usbip-win2)) | The driver all three depend on | Pin known-good versions. 0.9.7.8 can crash Windows (DSZ blocks it) |

## 4. Decision: **build on DSZ**
- **MIT**, so ideas from all three can be used without license problems. DualSenseClient's GPL code is borrowed as *ideas only*.
- A **native, lightweight hot path** (Rust, in-process USB/IP, high-resolution timers). DS-HapticLink's Python hot path and DualSenseClient's .NET plus Avalonia stack are heavier.
- It already has the **most complete feature set** and the **best crash hygiene**. Its gaps were process (no CI) and one installer bug, both fixed in the branch.
- Starting fresh would mean rebuilding about 38k lines of protocol work that DSZ already has tests for.

## 5. Changes made (branch `audit/hardening`, commit `2dadfcb`)
1. **Z1 fixed**: driver installers download into a new per-install folder and stay **locked against writes and deletes** from hashing until the elevated process starts.
2. **Z2 fixed**: the HidHide installer is now pinned by **SHA-256 and** signer (both must pass).
3. **Z4 fixed**: the USB/IP pre-import wait is cut from 5 s to 1 s.
4. **Z5 fixed**: **CI** runs Linux protocol tests, and on Windows builds, tests, publishes a SHA-256, and adds GitHub build-provenance attestation on tags.
5. README: hardware matrix (including your CFI-ZCT2W / BDM-060R) and a local-interfaces security note. Added `docs/hardware-test-plan.md`.

⚠️ The app crate **wasn't compiled here** (the Windows Rust target couldn't be downloaded). The first CI run on your fork is the compile check.

## 6. Phased build plan (DSZ fork)
| Phase | Goal | Work | Done when |
|---|---|---|---|
| **0. Baseline** (now) | Your fork builds | Push the branch, let CI build, download the artifact, run `docs/hardware-test-plan.md` on your PC | CI green; tests 1–9 and 11 recorded for BDM-060R (test 10, battery, parked 2026-11-09; due before Phase 3) |
| **1. Harden** | Close the remaining findings | Revocation checks (Z2 rest); optional UDP mod API off-by-default toggle (Z3); clippy `unwrap_used` outside tests (Z7) | No open Medium findings |
| **2. BDM-060 support** | Your board first-class | Log board/firmware from feature `0x20`/`0x22`; **borrow DS-HapticLink's cable-profile capture**: on first USB connect, save the real descriptors and feature reports and use them for the virtual pad | The virtual pad reports the real model; tests 4–7 pass |
| **3. Transport A/B** | Best haptics over BT | **Borrow DualSenseClient's idea**: an experimental Opus-in-`0x36` haptics mode alongside the raw `0x32` PCM; measure gaps and battery on BDM-060 | Data-backed default per board |
| **4. Fewer drivers** | Simpler install | **Borrow DS-HapticLink's no-HidHide mode** as an option: exclusive open after a device restart through a narrowly scoped elevated task, with logs kept in admin-only folders (avoiding H1) | Works without HidHide installed |
| **5. Per-app audio** | Cleaner audio-to-haptics | **Borrow DualSenseClient's idea**: Windows 11 per-process loopback as a haptics source | Choose a single app as the source |
| **6. Release** | Trustworthy builds | Tagged releases from CI only, with attestation; changelog; issue templates; offer the hardening upstream to MaximumFreak as a PR | First signed-provenance release |

---

# Audit 1: DSZ (DualSense Zen), MaximumFreak/dsz @ 43da7a1 (0.4.5, 2026-11-02)

## Snapshot
| | |
|---|---|
| Language | Rust (about 38.5k lines across `crates/dsz` and `crates/ds-proto`), plus vendored libopus and a patched eframe |
| History | 18 commits, 2026-10-09 → 2026-11-02, single author. Commits are squashed release drops |
| License | MIT. GPL-3.0 VIIPER is deliberately avoided so the project stays MIT |
| Tests | 285 `#[test]` total. The **ds-proto crate: 62/62 pass on Linux** (run here). The app crate is Windows-only and was not built here (the sandbox can't download the Rust Windows target) |
| CI | **None**. `.github` holds only FUNDING.yml. Releases are built on the author's machine and are not code-signed |

## How it works
```
Real DualSense ──BT HID (0x31 in/out, 0x32 haptics, 0x39 speaker+Opus)──► DSZ
DSZ ──in-process USB/IP server on 127.0.0.1:<random>──► usbip-win2 (UDE) ──► virtual wired DualSense
Game ──HID output + 4-ch USB Audio──► virtual DualSense ──► DSZ mixes ch3/4 → 3 kHz 8-bit PCM → BT 0x32 every 10.67 ms
HidHide hides the real pad from games; DSZ whitelists itself
```
- `virt/usbip/*`: its **own USB/IP server**, built from the USB/IP spec and the USB Audio Class spec. No external process, no IPC hop.
- `haptics/streamer.rs`: builds the `0x32` and `0x39` reports. It handles game haptics, audio-to-haptics, rumble-to-haptics, a limiter, and Opus speaker audio with a 16:15 resampler.
- `dsdetect.rs` + `games.rs`: scan a game's files for DualSense support and **switch the profile automatically** when the game starts.
- `platform.rs`: `timeBeginPeriod(1)` plus high-resolution waitable timers for packet pacing.

## Unique and clever
1. The USB/IP server and the USB Audio Class device run inside the app, which gives the lowest latency and nothing extra to install beyond the drivers.
2. Automatic native-DualSense game detection with per-game profile switching.
3. The haptics level can go past 100% with a peak limiter, which keeps faint effects from being rounded away in 8-bit BT samples.
4. Opus speaker audio. The author measured that the controller plays two frames per tick and built a 16:15 resampler to match.
5. Quick Charge: drops the BT link when the same MAC address appears over USB.
6. HidHide changes are journaled and **undone after a crash** on the next start.
7. Driver installers are downloaded and checked (usbip-win2 by **pinned SHA-256**, HidHide by Authenticode signer) before they run elevated.
8. Self-update never runs a downloaded exe. The browser downloads it, then it's checked against GitHub's published SHA-256 digest. This was done specifically to avoid antivirus "downloader" heuristics.
9. Diagnostic probe modes (`--probe-virtual ...`) make it possible to test each layer from the command line.

## Findings
| # | Severity | Where | Issue | Suggested fix |
|---|---|---|---|---|
| Z1 | **Medium** | `install.rs::run` | **TOCTOU (check-then-use gap) before elevation.** The installer is downloaded to a fixed path (`%TEMP%\DSZ\<name>`), hashed or signature-checked, and then run **by path** as administrator. Any same-user process can swap the file between the check and `ShellExecuteExW("runas")`. The UAC prompt then shows a valid-looking name while running attacker code. | Download to a fresh random directory, open the file with `FILE_SHARE_READ` only (no write or delete sharing), verify through that handle, and keep the handle open until the elevated process has started. |
| Z2 | Low | `install.rs::signer` | `WTD_REVOKE_NONE`: a revoked HidHide signing certificate would still pass. The signer is matched by display name only. | Use `WTD_REVOKE_WHOLECHAIN` (with an offline fallback), or pin the HidHide installer by SHA-256 the same way usbip-win2 is pinned. |
| Z3 | Low | `udp.rs`, `dsu.rs` | The UDP mod API (`127.0.0.1:6969`) and the DSU server accept commands from any local process with no authentication. This is by design, for DSX-mod compatibility. | Document it. Optionally add a toggle so it is off unless enabled. |
| Z4 | Low | `virt/usbip/mod.rs::session` | The accept loop handles one client at a time with a 5 s read timeout. A local process that connects and sends nothing stalls the virtual pad's attach for 5 s per attempt. | Cut the pre-import timeout to about 500 ms, or require the expected busid within one read. |
| Z5 | Process | repo | No CI. Releases are unsigned and built locally, so it can't be checked that a release matches the source. | Add a GitHub Actions Windows build that attaches the exe plus its SHA-256 (artifact attestation). |
| Z6 | Process | README | Says "early release (0.3)" while the version is 0.4.5, and "tested on one PC with a DualSense Edge". It has never been tested on a **BDM-060 / CFI-ZCT2** board like yours. | Run the hardware test matrix (see plan). |
| Z7 | Info | code | 132 `unsafe` blocks (Win32 FFI) and 250 `unwrap()`s. The ones spot-checked are on locks or test code. A panic hook and `catch_unwind` exist around device workers. | Track with clippy `unwrap_used` outside tests. |

Spot-checks that **passed**: USB/IP parser bounds (transfer ≤ 1 MiB, ≤ 1024 ISO packets), loopback-only listener, capture packet sizing capped, HidHide crash recovery, SHA-256-pinned usbip-win2 download, update digest check plus `MZ` header check.

## Verdict
The most complete and most carefully engineered of the three. Its weaknesses are process (no CI, one author, one tested PC), plus one real elevation-path bug (Z1).

---

# Audit 2: DS-HapticLink, SURTUR-00/DS-HapticLink @ 4b8a984 (0.4.10, 2025-10-25)

## Snapshot
| | |
|---|---|
| Language | Python 3.14 (about 5k lines), with a WebView2 UI in HTML/JS. Packaged with Nuitka |
| History | 9 commits, all on one day (squashed drops). 0.4.9 was the "first public release" |
| License | MIT |
| Tests | 74 tests. **65 pass on Linux** with a Windows API shim; the 4 failures and 5 skips are Windows-only paths (shim artifacts, not bugs) |
| CI | **None** |
| Dependencies | hidapi, PyOgg (Opus), pywebview, pythonnet, pystray, pillow, plus the external `usbip.exe` CLI |

## How it works
```
Real DualSense ──BT HID──► hidapi, opened exclusively (no HidHide)
                            ▲ the device is disable/enable-restarted by an elevated scheduled task so Steam/Windows let go
Python asyncio USB/IP server ──► `usbip.exe attach` (usbip-win2) ──► virtual wired DualSense
Game 4-ch 48 kHz audio ──► FIR low-pass + decimate ÷16 → 3 kHz int8 → BT haptics report; speaker via Opus (PyOgg)
```

## Unique and clever
1. **No HidHide.** It gets exclusive access by restarting the device node (`CM_Query_And_Remove_SubTreeW` with `CM_REMOVE_DISABLE`, then `CM_Enable_DevNode`) and grabbing the HID handle first. One less driver to install.
2. **A narrowly scoped elevated helper.** A scheduled task (one UAC prompt, ever) can only restart device instance IDs that match a **strict DualSense regex** (`VID_054C&PID_0CE6/0DF2`, BT or USB). The task exe is copied to `Program Files`, so the user can't modify what runs elevated.
3. **The installer TOCTOU is already fixed.** It verifies the bytes in memory (pinned SHA-256), writes them to an admin-only folder, checks Authenticode there, and runs from there. This is the fix DSZ is missing (DSZ Z1).
4. **Optional cable profile capture.** It reads a controller's real USB descriptors and feature reports over a cable so the virtual pad can match unusual models. It also reads the **live** feature reports (0x05 calibration, 0x09/0x0B MAC addresses, 0x20/0x22 firmware) over BT, so the virtual pad reports the real controller's identity.
5. It re-enables controllers that were left disabled (Device Manager code 22) after a blocked restart. This is a real-world bug the author hit and fixed.
6. Opus speaker and headphone-jack support plus the mic, with a well-thought-out time-real-time setup (`timeBeginPeriod(1)`, thread priorities).

## Findings
| # | Severity | Where | Issue | Suggested fix |
|---|---|---|---|---|
| H1 | **Medium** | `bridge/devrestart.py::_task_main`, `helper/dsh_helper.py::Progress/log` | **An elevated process writes into a folder the user can write to.** `C:\ProgramData\DSHapticLink\` inherits ProgramData's "Users can create" ACL. The elevated task appends to `restart.log` and *rewrites* it (truncation at 200 KB), and the elevated installer writes `setup.log` and `setup_status.json` via `os.replace`. A same-user medium-integrity process can pre-plant a junction or object-manager symlink. That's the classic path to an **arbitrary file write or overwrite as admin**, which amounts to a UAC bypass. | Have the elevated parts log only inside `Program Files\DSHapticLink Helper\logs` (admin-only), or create the ProgramData folder with an explicit ACL (Admins/SYSTEM full, Users read) and refuse to follow reparse points (`FILE_FLAG_OPEN_REPARSE_POINT` plus an attribute check). |
| H2 | Low | `devrestart.py` | Any local process can `schtasks /run` the task and drop a request file. Impact is limited to restarting DualSense devices because of the regex allow-list. This is acceptable, but should be documented. | Document it. Optionally check that the request file's owner is the installing user. |
| H3 | Low/compat | `bridge/profiles.py` | The built-in profile is a capture of **one reference PID 0x0CE6 controller**. The CFI-ZCT2 (BDM-060) has never been tested, and any descriptor change in the new board would need the "cable profile" capture. | Add BDM-060 to the hardware test matrix. Capture a cable profile on first USB connect automatically. |
| H4 | Low | `helper/dsh_helper.py::authenticode_ok` | `WTD_REVOKE_NONE` (the same as DSZ Z2). The pinned SHA-256 makes this mostly moot. | Optional. |
| H5 | Perf/robustness | `bridge/haptics.py`, asyncio USB/IP | The isochronous audio path runs in pure-Python asyncio, and timing quality depends on the GIL plus `time.sleep` with a 1 ms timer. The author already documents micro-gaps in continuous-haptics games. | Moving the hot path to native code (Rust or C) is the long-term fix. |
| H6 | Process | repo | No CI. One-day history. Release zips are Nuitka-built locally (antivirus false positives are documented). Source comments and the helper's default `--lang` are Spanish. | Add CI, reproducible builds, and English comments if contributors are wanted. |
| H7 | Compat | design | It relies on the external `usbip.exe` CLI being on disk at a fixed path (`C:\Program Files\USBip`). | Talk to the usbip-win2 driver IOCTL directly, as DSZ does. |

## Verdict
The cleverest *installation/permission model* and the leanest dependency set (no HidHide). But it's the youngest project, the hot path is in Python, and there's one real elevated-write issue (H1).

---

# Audit 3: DualSenseClient, DualSenseClient/DualSenseClient @ 3d0377b (2026-11-04)

## Snapshot
| | |
|---|---|
| Language | C# / .NET 10, Avalonia UI (Windows **and Linux**). About 65k lines of source plus 47k lines of tests |
| History | 332 commits, 2025-11 → 2026-11, steady activity (125 commits in Aug 2026). Has an issue template, docs site, translations workflow |
| License | **GPL-3.0** |
| Tests | **1,490** xUnit facts/theories, run in the release workflow (`dotnet test`). They could not be run here: the sandbox can't reach the .NET SDK or NuGet |
| CI | Yes: Windows and Linux publish jobs, release, docs, changelog, translations |
| Dependencies | Avalonia 12, FluentAvalonia, CommunityToolkit.Mvvm, Concentus (Opus), SoundFlow, Nefarius.Drivers.HidHide, Tmds.DBus (Linux BT) |

## How it works
```
Real DualSense ──HID (USB or BT)──► Controllers/DualSense ──► Opus-encoded haptics + speaker (BT reports 0x36 combined / 0x32 init-prime)
Emulation/UsbIp: managed USB/IP server on 127.0.0.1:3241/3242 ──► attach via VHCI IOCTL (fallback: usbip.exe) ──► virtual DS/DS4/X360
EmulationDualSenseAudioCapture: game audio from the virtual DualSense's endpoint ──► forwarder ──► physical pad over BT
HidHide (optional) hides the physical pad; AutoProfileCoordinator switches profiles per running exe
```

## Unique and clever
1. **Cross-platform**: the same app on Windows and Linux (Linux uses the kernel `usbip` plus udev rules).
2. **Pluggable attach strategies**: direct VHCI IOCTL first, `usbip.exe` as a fallback. This is more robust than DS-HapticLink's CLI-only approach.
3. **Opus-encoded haptics** inside the combined `0x36` BT report. This is a different transport from DSZ's raw 3 kHz PCM in `0x32`, and should be benchmarked on the BDM-060 board.
4. **Audio source routing**: the virtual controller's endpoint, a system loopback, or **per-application loopback** (Windows 11 process loopback).
5. By far the **largest test suite** (about 1.5k tests), an architecture with dependency injection and separate projects, docs, and i18n infrastructure. This is the most "team-ready" codebase.
6. Its credits list the prior art it builds on (SAxense, DS5Dongle, dualsense-bt-haptics, the ViGEmBus simple_ds5 fork, LinuxAudio4Dualsense5). It's a good map of the whole ecosystem.

## Findings
| # | Severity | Where | Issue | Suggested fix |
|---|---|---|---|---|
| C1 | **Medium** | `Core/Utilities/UpdateInstaller.cs::VerifyHashAsync` | **The update check passes the update when the `.sha256` sidecar returns 404** (it returns `true` "for older releases"). If the sidecar is missing or removed, an unverified exe is swapped in and launched. The hash also comes from the same origin as the asset, so it only checks integrity, not authenticity. | Fail closed on 404 for all releases from now on. Verify against GitHub's API `digest` field (as DSZ does), or sign the releases. |
| C2 | Low/UX | `HidHide/HidHideService.cs`, `AutoProfileCoordinator` | Hiding is applied directly to the HidHide blocklist, with **no crash journal**. If the app crashes or is killed while it has auto-hidden a controller, the real pad stays invisible to every other app and game until DualSenseClient runs again. The app also adds itself to the HidHide allow-list permanently. | Record which hides the app applied and undo them on the next start (as DSZ does). Offer an "unhide everything I hid" action on exit. |
| C3 | Low | `Emulation/UsbIp/UsbIpServer.cs` | **Fixed** loopback ports 3241/3242. Another local process that binds first breaks emulation, and could squat the port and present its own device to the VHCI driver. | Use an ephemeral port (`:0`) and pass it to the attach call, as DSZ does. |
| C4 | Low | design | Its BT game-haptics support isn't mentioned in the README's feature table, and the code path is spread across a 1,400-line `DualSenseAudioForwarder.cs`. It's hard to tell from the outside whether game haptics are fully supported. | Document it and add a probe/diagnostic mode. |
| C5 | Licensing | repo | **GPL-3.0.** Any code copied from it makes the derived work GPL. That affects a project that wants to stay MIT, such as one based on DSZ. | Borrow *ideas* only, or keep any new project GPL. |
| C6 | Info | code | 11 `async void` handlers (all UI event handlers, which is acceptable). Self-update downloads and launches an exe, which DSZ notes triggers antivirus "downloader" heuristics. | Optional: use DSZ's browser-download update flow. |

## Verdict
The most mature *engineering process* (CI, tests, docs, steady commits, cross-platform). But it carries the heaviest runtime (.NET 10 plus Avalonia), it's GPL, and its update path fails open (C1).
