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
| 3 | Connect | 2026-11-07 | ✅ Pass | BT connect logged 73 ms after start (looked instant). Log: `connected DualSense over Bluetooth, MAC 0C:27:56:B7:C3:3D, firmware 6.41 · Mar 17 2026`. Board revision not logged (gap → Phase 2) |
| 4 | Virtual DualSense | 2026-11-07 | ✅ Pass | Marvel's Spider-Man 2 (Epic, Steam closed). PS prompts; adaptive triggers while swinging; haptics in menus, footsteps, swinging; speaker plays beeps/calls (call voices on PC speakers, game routing). See issues I1, I2 |
| 5 | Game haptics | | | |
| 6 | Speaker | | | |
| 7 | Rumble fallback | | | |
| 8 | Crash recovery | | | |
| 9 | Quick Charge | | | |
| 10 | Battery | | | |
| 11 | Latency | | | |

## Issues found

| # | Seen in | Issue | Evidence | Status |
|---|---|---|---|---|
| I1 | Test 4 | Two mid-game BT drops (~50 min session) | `dsz.log` 22:01 and 22:15 UTC: `0x31` writes time out (250 ms each) for ~34 s, then "DualSense dropped the link by itself (battery 90 %)". First reconnected in 1 s, second needed the PS button. A third disconnect at 22:31 was DSZ's 15-min idle-off after the game closed (expected). Adapter: onboard MediaTek RZ616 (MT7922), Wi-Fi unused (Ethernet), BT "allow turn off to save power" on | Investigating: antenna/range, BT power saving, other BT audio |
| I2 | Test 4 | Occasional faint crackle from the controller speaker | User report; no underrun warnings in the log | Check in Test 6, likely same radio pressure as I1 |