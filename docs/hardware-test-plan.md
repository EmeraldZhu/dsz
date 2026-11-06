# Hardware test plan: DualSense CFI-ZCT2W (BDM-060R) over Bluetooth

These are the tests that can't run in CI. Make a Windows restore point first. Close DSX, DS4Windows and Steam before each test unless the step says otherwise. Note the results in an issue.

| # | Test | How | Pass |
|---|---|---|---|
| 1 | Driver install (Z1/Z2 fix) | Settings → Drivers → install usbip-win2, then HidHide | Each shows "Verifying" then the admin prompt. `%TEMP%` holds a `DSZ-install-…` folder only while installing; it's gone afterwards |
| 2 | Tamper check | During install, try to delete or rename the downloaded exe in its `DSZ-install-…` folder | Windows refuses ("file in use") until the installer closes |
| 3 | Connect | Pair over BT, start DSZ | The controller appears within about 1 s. The log shows firmware and board info |
| 4 | Virtual DualSense | Profiles → DualSense, launch a native-DualSense game | The game shows PS prompts, and **adaptive triggers** work over BT |
| 5 | Game haptics | Same game, in a scene with constant haptics (rain, engine) | Smooth haptics with no ticking or gaps. The log has no `underrun` warnings |
| 6 | Speaker | A game that uses the controller speaker | Clear audio. Note any gaps (adapter pauses) |
| 7 | Rumble fallback | A non-native game with an Xbox 360 profile | Rumble is converted to haptics |
| 8 | Crash recovery | End `dsz.exe` in Task Manager while the virtual pad is up, then start DSZ again | The real controller is visible to other apps again (HidHide undone) |
| 9 | Quick Charge | Plug the USB cable in while on BT | One controller in games, with no duplicates |
| 10 | Battery | 1 h of play with haptics over BT | Note the battery % used |
| 11 | Latency | `--probe-virtual mic-listen --full` | Note the adapter pause stats |
