# Hardware results: Phase 0 baseline

Plan: `docs/hardware-test-plan.md`.

| | |
|---|---|
| Controller | Sony DualSense CFI-ZCT2W (Grey Camouflage), board BDM-060R |
| Firmware | 6.41 · Mar 17 2026 (DSZ log, USB) |
| PC | Windows 11 Pro 10.0.26300, Ryzen 7 7800X3D, RTX 5070 Ti, ASRock B850M Pro-A WiFi (onboard BT) |
| DSZ build | Tests 1–4: 0.4.5 + hardening (`audit/hardening` @ 9e97f23). Tests 5+: 0.4.6 + hardening (`audit/hardening` @ 57f8f89, upstream v0.4.6 merged). Local `cargo build --release -p dsz --locked` |
| Drivers | usbip-win2 0.9.8.1, HidHide 1.5.230 (as pinned in `install.rs`) |

Note: the driver install buttons are on the **Virtual controller** page ("Drivers on this PC"), not Settings.

| # | Test | Date | Result | Notes |
|---|---|---|---|---|
| 1 | Driver install (Z1/Z2) | 2026-11-07 | ✅ Pass | Both showed Downloading → Checking → UAC; rows went to "ready"; no `DSZ-install-*` folder left (verified). Installed: USBip 0.9.8.1, HidHide 1.5.230 |
| 2 | Tamper check | 2026-11-07 | ✅ Pass | Delete and rename of the usbip-win2 installer refused while UAC/installer open |
| 3 | Connect | 2026-11-07 | ✅ Pass | BT connect logged 73 ms after start (looked instant). Log: `connected DualSense over Bluetooth, MAC 0C:27:56:B7:C3:3D, firmware 6.41 · Mar 17 2026`. Board revision not logged (gap → Phase 2) |
| 4 | Virtual DualSense | 2026-11-07 | ✅ Pass | Marvel's Spider-Man 2 (Epic, Steam closed). PS prompts; adaptive triggers while swinging; haptics in menus, footsteps, swinging; speaker plays beeps/calls (call voices on PC speakers, game routing). See issues I1, I2 |
| 5 | Game haptics | 2026-11-07 | ✅ Pass | 0.4.6 build. Spider-Man 2, ~32 min of game audio (23:20–23:52 UTC). No ticking/gaps reported; log has zero WARN/ERROR, no underruns, no disconnects. Overlay/crashpad no longer trigger profile switches (0.4.6 fix confirmed) |
| 6 | Speaker | 2026-11-07 | ⚠️ Works, with defects | 0.4.6. Mostly clear in Spider-Man 2 (calls, beeps); occasional crackle and short gaps in game **and** in DSZ's Test speaker, so not game-specific. 7 Test speaker runs (DSZ measures via the controller mic): dropouts 0/1/2/1/0/0/0, clicks after the start transient 0–24 per 4.9 s run; mic uplink lost 1 of ~3,420 packets. See I2 |
| 7 | Rumble fallback | | | |
| 8 | Crash recovery | | | |
| 9 | Quick Charge | | | |
| 10 | Battery | | | |
| 11 | Latency | | | |

## Issues found

| # | Seen in | Issue | Evidence | Status |
|---|---|---|---|---|
| I1 | Test 4 | Two mid-game BT drops (~50 min session) | `dsz.log` 22:01 and 22:15 UTC: `0x31` writes time out (250 ms each) for ~34 s, then "DualSense dropped the link by itself (battery 90 %)". First reconnected in 1 s, second needed the PS button. A third disconnect at 22:31 was DSZ's 15-min idle-off after the game closed (expected). Adapter: onboard MediaTek RZ616 (MT7922), Wi-Fi unused (Ethernet), BT "allow turn off to save power" on | Mitigations applied 2026-11-07: both antennas confirmed attached; BT power saving off; MediaTek BT driver 1.1147.0.613 installed; speakers are wired (Edifier M206BT on aux), so no other BT audio. Test 5: no drops in ~32 min (previous session: 2 drops in ~50 min). Keep watching in tests 6 and 10 |
| I2 | Test 4 | Occasional faint crackle from the controller speaker | User report; no underrun warnings in the log | Confirmed in Test 6 (measured, see test 6 row). Uplink is clean, so the loss is on the PC→controller speaker stream (`0x39`). Quantify adapter pauses in Test 11; fix work belongs to Phase 3 (transport) |