# CLAUDE.md

Fork of MaximumFreak/dsz (DSZ, Rust, MIT): wireless DualSense haptics / adaptive triggers over Bluetooth via a virtual wired DualSense (usbip-win2 + HidHide).
**Read `AGENTS.md` first** — its hardware, i18n, profile and PR rules are binding. Audit + plan: `docs/AUDIT.md`. Hardware tests: `docs/hardware-test-plan.md`, results in `docs/hardware-results.md`.

## Commands (PowerShell 5: chain with `;`, not `&&`)
- Build: `cargo build --release -p dsz --locked` → `target\release\dsz.exe`
- Test: `cargo test --workspace --release --locked` (no new warnings)
- Protocol only (any OS): `cargo test -p ds-proto --release`
- UI snapshot (debug build, no controller): `$env:DSZ_SNAPSHOT='out.bmp'; $env:DSZ_DATA_DIR='tmp'; cargo run -p dsz -- --page <name>`
- Probe without input injection: `dsz.exe --probe-virtual ...`
- CI: `.github/workflows/ci.yml`; check with `gh run list` / `gh run view <id> --log-failed`
- No rustfmt over files; format only touched lines.

## Architecture
- `crates/ds-proto`: protocol only (input/output reports, BT `0x32` haptics, `0x39` speaker, CRCs, trigger effects). No Windows code.
- `crates/dsz/src`: `device.rs` (per-controller threads), `pipeline.rs` (input → mappings), `composer.rs` (output state), `haptics/` (BT audio stream), `virt/usbip/` (in-process USB/IP server + descriptors), `install.rs` (driver installers, elevated), `update.rs`, `udp.rs`/`dsu.rs` (local mod APIs), `ui/` (egui), `i18n.rs` + `locales/`.

## Owner / preferences
- Justus (EmeraldZhu), Windows 11, DualSense CFI-ZCT2W, board BDM-060R. AI-assisted: Claude writes/runs code, Justus tests on hardware.
- Short, dense answers in clear sections. Name a tool → give its download/source link.
- Ask before anything irreversible: driver installs, deleting files, force-push, any push.
- Commit after each logical step on the working branch; don't push without asking.

## Phase plan (details in `docs/AUDIT.md` §6)
0. **Baseline** — builds locally + CI green; hardware tests 1–11 logged. *Don't start Phase 1 until done.*
1. **Harden** — revocation checks (Z2), UDP mod API toggle off by default (Z3), clippy `unwrap_used` outside tests (Z7).
2. **BDM-060 support** — log board/firmware (`0x20`/`0x22`); capture real USB descriptors for the virtual pad.
3. **Transport A/B** — experimental Opus-in-`0x36` haptics vs raw `0x32` PCM; measure.
4. **Fewer drivers** — optional no-HidHide mode (exclusive open after device restart).
5. **Per-app audio** — Win11 process loopback as haptics source.
6. **Release** — CI-only tagged releases with attestation; upstream PR.
