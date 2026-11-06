# Hardware results: Phase 0 baseline

Plan: `docs/hardware-test-plan.md`.

| | |
|---|---|
| Controller | Sony DualSense CFI-ZCT2W (Grey Camouflage), board BDM-060R |
| Firmware | 6.41 · Mar 17 2026 (DSZ log, USB) |
| PC | Windows 11 Pro 10.0.26300, Ryzen 7 7800X3D, RTX 5070 Ti, ASRock B850M Pro-A WiFi (onboard BT) |
| DSZ build | `audit/hardening` @ 9e97f23, local `cargo build --release -p dsz --locked` |
| Drivers | usbip-win2 0.9.8.1, HidHide 1.5.230 (as pinned in `install.rs`) |

Note: the driver install buttons are on the **Virtual controller** page ("Drivers on this PC"), not Settings.

| # | Test | Date | Result | Notes |
|---|---|---|---|---|
| 1 | Driver install (Z1/Z2) | 2026-11-07 | ✅ Pass | Both showed Downloading → Checking → UAC; rows went to "ready"; no `DSZ-install-*` folder left (verified). Installed: USBip 0.9.8.1, HidHide 1.5.230 |
| 2 | Tamper check | 2026-11-07 | ✅ Pass | Delete and rename of the usbip-win2 installer refused while UAC/installer open |
| 3 | Connect | | | |
| 4 | Virtual DualSense | | | |
| 5 | Game haptics | | | |
| 6 | Speaker | | | |
| 7 | Rumble fallback | | | |
| 8 | Crash recovery | | | |
| 9 | Quick Charge | | | |
| 10 | Battery | | | |
| 11 | Latency | | | |
