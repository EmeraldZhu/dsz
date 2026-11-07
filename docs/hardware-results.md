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
| 7 | Rumble fallback | 2026-11-07 | ✅ Pass (weak) | The Finals (Steam), "Other games" = Xbox 360. Fallback switched correctly (`discovery.exe has no DualSense support: virtual Xbox 360`, back to DualSense on exit). Rumble felt but weak; explosions often not felt. Profile had "Rumble as haptics" off, so rumble went to the controller's own rumble emulation (speaker/haptic stream was off in Xbox mode). Re-test 00:34–00:44 UTC with "Rumble as haptics" on: stronger, still below expectations; user calls it a pass. Caveat: input was lost in the fallback (I6), so the user added a rule discovery.exe → "Xbox" profile at 00:37:03; that profile has rumble as haptics off, so the last 7 min ran without it. See I4 |
| 8 | Crash recovery | | | |
| 9 | Quick Charge | | | |
| 10 | Battery | | | |
| 11 | Latency | | | |

## Issues found

| # | Seen in | Issue | Evidence | Status |
|---|---|---|---|---|
| I1 | Test 4 | Two mid-game BT drops (~50 min session) | `dsz.log` 22:01 and 22:15 UTC: `0x31` writes time out (250 ms each) for ~34 s, then "DualSense dropped the link by itself (battery 90 %)". First reconnected in 1 s, second needed the PS button. A third disconnect at 22:31 was DSZ's 15-min idle-off after the game closed (expected). Adapter: onboard MediaTek RZ616 (MT7922), Wi-Fi unused (Ethernet), BT "allow turn off to save power" on | Mitigations applied 2026-11-07: both antennas confirmed attached; BT power saving off; MediaTek BT driver 1.1147.0.613 installed; speakers are wired (Edifier M206BT on aux), so no other BT audio. Test 5: no drops in ~32 min (previous session: 2 drops in ~50 min). Keep watching in tests 6 and 10 |
| I2 | Test 4 | Occasional faint crackle from the controller speaker | User report; no underrun warnings in the log | Confirmed in Test 6 (measured, see test 6 row). Uplink is clean, so the loss is on the PC→controller speaker stream (`0x39`). Quantify adapter pauses in Test 11; fix work belongs to Phase 3 (transport) || I3 | Test 7 prep | The "Other games" fallback pad only applies to paths with `\steamapps\common\`, `\gog games\` or `\xboxgames\` (`games.rs::is_likely_game`). GOG Galaxy's own folder (`...\GOG Galaxy\Games\`), Epic, EA, Ubisoft, Battle.net and Rockstar installs never get the fallback; they keep the profile's pad unless a per-game rule exists | Code read; user's GOG path is `D:\Games\GOG Galaxy\Games\` | Open. Candidate fix (off-by-default-safe): also match `\gog galaxy\games\` and Epic's manifest folders. Phase 1 || I4 | Test 7 | Rumble in the Xbox 360 fallback feels weak and misses big events | User report; DualSense profile `rumble_as_haptics: false`, so DSZ used the controller's own rumble emulation instead of its haptic stream | Improved with "Rumble as haptics" on (DualSense profile). Still weaker than expected; tune with "Haptic rumble strength" (up to 200 %) |
| I5 | Test 7 prep | DSZ's DualSense detection may miss games that support it on PC | Offline scan with dsdetect's markers said no for GTA V Enhanced, Split Fiction, The Finals. GTA V Enhanced's March 2025 PC update lists DualSense adaptive triggers, so at least that one is a miss. Confirm with DSZ's own verdict when each game runs | Open. Workaround: a per-game rule in Profiles & games. Phase 2 candidate || I6 | Test 7 re-test | Steam game + Xbox 360 fallback: Steam showed repeated "controller connected/disconnected" popups, then the game stopped seeing any input, even in menus | User report. Log 00:34:35–00:35:05 UTC: virtual DualSense removed, helper Xbox pad unplugged, virtual Xbox 360 removed and re-attached on port 1, helper re-plugged on port 2, helper unplugged 30 s later; each switch re-plugs pads. Steam was started by the game launch (Steam Input state for The Finals unknown). Worked around with a per-game rule to the "Xbox" profile | Open. Reproduce with Steam Input off for the game vs on; check whether the helper pad churn is what Steam reacts to. Phase 1 candidate |