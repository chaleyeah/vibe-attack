# Domain Pitfalls: Linux Voice-Macro / Input-Injection for Gaming on Wayland

**Project:** hd-linux-voice
**Domain:** Linux desktop voice-control + input injection for gaming (Wayland-first, Helldivers 2 target)
**Researched:** 2026-04-21
**Overall confidence:** HIGH (Wayland/uinput), MEDIUM (anti-cheat risk), HIGH (audio), HIGH (distribution), HIGH (ASR latency)

---

## Severity Legend

| Symbol | Severity | Meaning |
|--------|----------|---------|
| 🔴 | **Critical** | Causes rewrite, blocks the core user journey, or gets users banned |
| 🟠 | **High** | Causes significant UX failures or requires painful architectural rework |
| 🟡 | **Medium** | Degrades experience; fixable without redesign |

---

## Critical Pitfalls

---

### 🔴 P-01 — Wayland Global Hotkey Dead Zone (Push-to-Talk Blocked by Game Focus)

**What goes wrong:**
Wayland's security model intentionally removes the X11 `XGrabKey` mechanism that allowed any application to register global hotkeys. When a game (via XWayland or native Wayland) holds keyboard focus and captures input, there is no standard mechanism for a background application to intercept the push-to-talk key. The macro app receives nothing while the game is in foreground.

**Why it happens:**
By design: Wayland compositors refuse to let arbitrary applications subscribe to global key events. The security model considers "other applications reading your keystrokes" a privacy violation. The `zwp_keyboard_shortcuts_inhibit_manager_v1` protocol goes the other direction (inhibit *compositor* shortcuts, not capture *application* input).

**Consequences:**
Push-to-talk is the primary interaction model for most users. If it silently stops working when the game takes focus, the entire product is broken during gameplay — the only time it matters.

**Warning signs (detect early):**
- Any approach using XLib's `XGrabKey` or GTK's global shortcut registration works on X11/XWayland wrapper windows but fails when tested inside a fullscreen Proton game
- Desktop-level hotkey libraries (e.g. `x11rb` global grab, `rdev`) work on GNOME/KDE desktops but produce no events during game

**Prevention:**
Read the physical keyboard device via `/dev/input/eventN` directly (evdev grab). This bypasses the Wayland security model because you are reading from the kernel input event device, not intercepting compositor-dispatched events. The daemon holds an exclusive `EVIOCGRAB` on the physical device, receives all key events regardless of focus, and re-injects non-PTT keys back via a separate uinput pass-through device.

**Phase/component to address:**
Input architecture phase — the push-to-talk subsystem must be designed around evdev grab from day one, not retrofitted. Document the `input` group membership requirement alongside the `uinput` group requirement.

---

### 🔴 P-02 — `/dev/uinput` Not Accessible by Default (Permissions Bootstrap)

**What goes wrong:**
`/dev/uinput` is owned by root with mode `crw-------` on most distributions out of the box. The `uinput` kernel module may not even be loaded. New installs silently fail to open the device — the app starts, speech is recognized, and nothing happens in the game.

**Why it happens:**
Linux distributions do not enable uinput for regular users by default for security reasons. This is correct behavior, but it means every installation requires two steps users won't know to perform: (1) load the kernel module, (2) configure udev and group membership.

**Consequences:**
Zero-install experience is broken. Users file bugs saying "nothing happens." Support burden is enormous without automated setup.

**Warning signs (detect early):**
- `open("/dev/uinput", O_WRONLY)` returns `ENOENT` (module not loaded) or `EACCES` (wrong permissions)
- Works when run as root but fails as a regular user
- Works on developer's machine (where setup was done manually) but fails on fresh VMs

**Prevention:**
Ship a post-install hook (systemd `.service` or `ExecStartPre=`) and udev rules file:
```
SUBSYSTEM=="misc", KERNEL=="uinput", GROUP="uinput", MODE="0660", TAG+="uaccess"
```
Install `uinput.conf` to `/etc/modules-load.d/` so the module loads at boot. The installer should also add the user to the `input` and `uinput` groups and print a clear message to log out and back in. Never silently continue when `/dev/uinput` can't be opened — fail fast with an actionable error linking to the setup documentation.

**Phase/component to address:**
Distribution / packaging phase. Also needs a runtime check in the app startup path with a human-readable error dialog.

---

### 🔴 P-03 — Flatpak Sandboxing Is Incompatible With uinput + evdev Grab

**What goes wrong:**
Flatpak's sandbox cannot grant `/dev/uinput` write access or `/dev/input/eventN` exclusive read access unless the *host* system already has the udev rules and group memberships configured — and even then, the `--device=all` flag required to expose these devices is a de-facto sandbox escape that portal reviewers reject for distribution. The microphone additionally requires `xdg-desktop-portal-pipewire` to be installed on the host.

**Why it happens:**
Flatpak is designed for GUI apps that access hardware through XDG portals (camera, microphone, location). Raw character device access (`/dev/uinput`, `/dev/input/*`) has no portal abstraction. The workaround (manual udev rules installed outside the sandbox) defeats the entire purpose of Flatpak for non-technical users.

**Consequences:**
If Flatpak is chosen as the primary distribution format, every user needs manual system configuration that Flatpak is supposed to eliminate. AppImage has a related but different problem: FUSE mounts are read-only, blocking `setcap` on the binary.

**Warning signs (detect early):**
- Flatpak app starts, requests microphone permission via portal, but `/dev/uinput` open fails inside container
- `--device=all` works in testing but is blocked by Flathub review policy
- Bug reports from users who installed via Flatpak but skipped the "install udev rules manually" step

**Prevention:**
Do **not** use Flatpak as the primary release format. Preferred distribution path for v1:
1. **Native packages** (`.deb` / `.rpm`) where the package `%post` script can install udev rules and load the kernel module
2. **AppImage** with a separate `install.sh` that handles privileged setup steps, clearly documented

Keep Flatpak as a future stretch goal if XDG portals eventually add uinput/evdev support.

**Phase/component to address:**
Distribution planning phase. Decide packaging format before building CI pipelines for it.

---

### 🔴 P-04 — XWayland Fullscreen Focus Race Causes Game to Minimize / Lose Input

**What goes wrong:**
Proton/Wine games run as XWayland clients. XWayland's handling of pointer and keyboard grabs interacts badly with some Wayland compositors during focus transitions, causing games to spontaneously:
- Self-minimize when another window (e.g. a daemon tray icon) briefly captures attention
- Lose relative mouse capture (FPS rotation stops working)
- Ignore all keyboard input until the game window is clicked again

The voice-macro daemon is an additional long-running process on the desktop — any UI it displays (system tray, notification, config window) can trigger this race.

**Why it happens:**
XWayland generates `FocusOut` events when Wayland sends input to a different client simultaneously. Fullscreen games use grab state tied to focus; losing focus breaks the grab, and games aren't designed to recover from mid-session focus loss.

**Consequences:**
User says a voice command → the daemon shows a notification → game minimizes → user must alt-tab back. The voice command may then fire in the wrong window. This is a game-breaking UX failure.

**Warning signs (detect early):**
- Running the daemon causes intermittent minimize during testing even without voice input
- Notifications or tray icon tooltip hover causes loss of game mouse capture
- Issue is compositor-specific: repros on Sway/Hyprland but not KDE Plasma

**Prevention:**
- Keep the daemon **headless by default** with no tray icon or floating windows unless explicitly opened by the user
- Use `gamescope` as the game compositor layer (strongly recommended for Helldivers 2) — gamescope isolates the game window from compositor focus events entirely
- Any config UI should be a separate process that the user explicitly launches/closes; it must not linger as a mapped window while gaming
- Test specifically by running the daemon + a fullscreen XWayland test app before testing with the real game

**Phase/component to address:**
UX/UI architecture phase. Core daemon design: headless daemon, separate UI process.

---

### 🔴 P-05 — End-to-End Latency Pipeline Blowout

**What goes wrong:**
Projects underestimate the cumulative latency: audio buffer fill → VAD trigger → recognition → post-processing → macro fire. Individual components seem fast in isolation. Combined, the pipeline can exceed 500–1500ms, which feels broken for real-time gaming ("I said Eagle Airstrike and it triggered two seconds later, in the wrong position").

**Why it happens:**
Each stage adds latency that compounds:
- Audio capture buffer: 50–200ms depending on block size
- VAD silence tail: 300–500ms of silence required to confirm end-of-speech
- Whisper inference: 100–800ms on CPU depending on model size and duration of audio
- Vosk streaming: lower latency but less accurate on game-specific vocabulary
- Key injection: negligible, but macro timing (hold/release) adds 100–300ms for strategems

Default configurations of Whisper (even `tiny`) are designed for transcription, not sub-200ms command detection.

**Warning signs (detect early):**
- Acceptable in quiet testing → unacceptable in actual gameplay
- Latency is consistent in testing but spikes during recognition of longer phrases
- VAD threshold tuned too conservatively (waits too long for silence before triggering)

**Prevention:**
- Instrument every stage with a timestamp log from day one
- Set a hard latency budget: **<300ms from end of last spoken syllable to first injected key event** for the default mode
- For Whisper: use `tiny` or `base.en` models; set short max audio duration; disable beam search (use greedy decoding)
- For Vosk: configure a restricted vocabulary/grammar for the command set; this dramatically speeds recognition
- Tune VAD aggressiveness and silence tail per use-case in the config
- Run the recognition model warm (pre-loaded, not cold-started per utterance)

**Phase/component to address:**
ASR backend selection phase AND a dedicated "latency profiling" milestone task before v1 release.

---

## High Pitfalls

---

### 🟠 P-06 — Unconstrained ASR Misrecognizes Gaming Vocabulary

**What goes wrong:**
General-purpose speech models (Whisper, Vosk default models) are trained on natural language corpora. Helldivers 2 strategem names ("Eagle Cluster Bomb", "Orbital Precision Strike", "Reinforce") are either ambiguous or out-of-distribution. The model transcribes something plausible that doesn't match any command → no action → user thinks the app is broken.

**Why it happens:**
Open vocabulary ASR finds the closest likely sequence given its training data. "Eagle Airstrike" might come back as "Eagle Air Strike" (variant spacing), "Eagle Air Strick", or something completely different in noisy conditions.

**Consequences:**
Core value proposition fails silently. Users see no error; the app seems to have ignored them. This erodes trust quickly.

**Warning signs (detect early):**
- WER (word error rate) on game-specific phrases is much higher than general English WER benchmarks
- Similar-sounding phrases trigger wrong commands

**Prevention:**
- Use a **constrained grammar / vocabulary list** approach, not open-ended transcription. For Vosk, configure `SetGrammar` with the exact recognized phrase set. This reduces the search space and dramatically improves accuracy and speed.
- For Whisper: use the `initial_prompt` parameter to bias toward expected terms, or implement a post-recognition matcher that maps recognized text → command via fuzzy/phonetic matching with a confidence threshold
- Allow per-command alternative phrases ("Reinforce" / "Reinforce call" / "call in support")
- Log what was recognized vs what matched; expose in debug mode

**Phase/component to address:**
ASR integration phase; command matching subsystem.

---

### 🟠 P-07 — Game Noise / Audio Bleed Causes VAD False Triggers

**What goes wrong:**
Game audio (explosions, voice lines, Discord call audio) leaks into the microphone — especially on headsets with poor isolation or open-back headphones. The VAD fires on game sounds. The recognition model then processes game audio and either produces garbled commands or silently fails. In the worst case, an in-game voice line accidentally triggers a strategem.

**Why it happens:**
On Linux, PipeWire/PulseAudio expose both physical microphone sources and "monitor" sources. An incorrectly selected input source (e.g. capturing from the game audio output monitor) will include all game sounds.

**Warning signs (detect early):**
- VAD fires intermittently when user is not speaking
- Recognized text during false triggers contains words from in-game dialogue

**Prevention:**
- Default to the physical microphone source, not any monitor source — filter out `monitor` sources in device enumeration
- Make the input device clearly configurable with a readable name (not ALSA card index)
- Implement a noise gate above the VAD to require sustained energy levels (not just any audio activity)
- Make VAD sensitivity configurable per-user
- Document headset/microphone recommendations prominently

**Phase/component to address:**
Audio capture subsystem; configuration/setup wizard.

---

### 🟠 P-08 — Anti-Cheat Detection of Virtual Input Devices

**What goes wrong:**
Anti-cheat systems can enumerate connected HID/input devices. A uinput-created virtual keyboard appears as an additional input device in `/proc/bus/input/devices` and in kernel input event lists. Some anti-cheat systems flag the presence of unexpected virtual input devices as suspicious, even if the inputs themselves are legitimate.

**Current risk for Helldivers 2:**
Helldivers 2 uses **nProtect GameGuard**, which runs in **userspace** under Proton (not kernel-level on Linux). Current community reports indicate it does not ban for Linux uinput usage, and the game works on Linux/Steam Deck. However:
- This is a soft guarantee — GameGuard behavior can change in updates
- Other games this project might expand to (BattlEye, EAC) carry higher risk
- Running as root to inject input is a much larger anti-cheat flag than using uinput with correct permissions

**Warning signs (detect early):**
- Game kicks to main menu after command injection
- Account flagged or banned (rare but catastrophic)
- Anti-cheat driver update causes regression

**Prevention:**
- Always use **user-level uinput** (never run the injection as root)
- Name the virtual device something neutral, not "voicemacro-bot-keyboard"
- Do not inject during loading screens or menus where timing looks inhuman (instant key sequences)
- Build in realistic key press dwell times (50–150ms) matching physical human input
- Monitor community reports after each GameGuard update; maintain a "tested versions" list in docs
- Design the architecture so input injection can be **disabled** without stopping recognition — useful if a specific game update breaks compatibility

**Phase/component to address:**
Input injection backend; game compatibility testing phase.

---

### 🟠 P-09 — uinput Key Repeat Double-Fire

**What goes wrong:**
When the `EV_REP` capability is advertised on a uinput virtual device AND userspace also sends its own repeat events, the kernel generates its own repeat events *in addition* to userspace-generated ones. The result is double key repeat — macro sequences fire twice or at double speed.

**Why it happens:**
The kernel automates `EV_REP` key repetition when the capability bit is set on a device. If the application also sends explicit repeat events, both fire simultaneously.

**Warning signs (detect early):**
- Strategem sequences produce double key presses or extra characters in any text field
- Only manifests with longer held keys, not single-press events

**Prevention:**
When creating the uinput virtual keyboard device, explicitly **do not set `EV_REP`** in the capabilities bitmask. Only report `EV_KEY` and `EV_SYN`. Send only explicit `KEY_PRESS` (value=1) and `KEY_RELEASE` (value=0) events followed by `SYN_REPORT`. Do not send `KEY_REPEAT` (value=2) events from userspace.

**Phase/component to address:**
uinput device creation code; tested in the key injection unit test suite.

---

### 🟠 P-10 — Strategem Macro Timing Too Fast for Game Input Buffer

**What goes wrong:**
Helldivers 2 strategems require specific directional key sequences (e.g. Down, Right, Up, Left for an Eagle). Injecting these at machine speed (microsecond intervals) causes the game to drop inputs — the input buffer doesn't process all events, or the game's frame rate creates a sampling window that misses some keys.

**Why it happens:**
Games sample input at their own frame rate (often 60Hz = 16.6ms per frame). An input event injected and released within the same frame may not register. Physical humans hold keys for 50–200ms naturally.

**Warning signs (detect early):**
- Strategems sometimes fail to call the correct result even though the recognized phrase was correct
- Failure rate is higher at high frame rates
- Manually pressing the sequence works but voice-injected sequence misses steps

**Prevention:**
- Default key dwell time: **80–120ms per directional key** for strategem sequences
- Default inter-key gap: **30–50ms** between key release and next key press
- Make these configurable per-profile and per-command
- Test with a running Helldivers 2 instance during development, not just a key-logger

**Phase/component to address:**
Macro execution engine; Helldivers 2 strategem pack validation.

---

### 🟠 P-11 — PipeWire/PulseAudio/ALSA Enumeration Fragmentation

**What goes wrong:**
Linux audio is in a transitional era: most systems use PipeWire (which also emulates PulseAudio), some run native PulseAudio, some use ALSA directly. The same hardware surfaces as different device names and indices across these backends. A hardcoded ALSA device name that works on the developer's machine fails on a user running PulseAudio, and vice versa.

Additionally:
- Bluetooth microphones require `xdg-desktop-portal-pipewire` to be installed for portal-based access
- PipeWire's float32le audio format negotiation fails with modules expecting integer formats
- Device indices can change between reboots (ALSA)

**Warning signs (detect early):**
- "It works on my machine" — developer uses PipeWire, tester uses PulseAudio
- Bluetooth headset mic works with other apps but not the voice-macro daemon
- Device selects the wrong microphone after OS update

**Prevention:**
- Use a library that abstracts audio backends: **CPAL** (Rust) handles PipeWire, PulseAudio, and ALSA through a unified interface
- Enumerate devices by name, not index; persist the selected device name in config
- Test on both a PipeWire system and a native PulseAudio system in CI
- Do not depend on any specific sample rate or format — accept what the device offers and resample if needed
- Document Bluetooth microphone requirements (portal must be installed)

**Phase/component to address:**
Audio capture subsystem. Treat backend abstraction as a first-class design constraint.

---

### 🟠 P-12 — AGPL License Compatibility of ASR Model Dependencies

**What goes wrong:**
The project is AGPL-3.0. Third-party libraries and models included in distribution must be AGPL-compatible. Speech model licenses are often separate from their inference engine licenses and may include non-commercial or "research only" restrictions that are incompatible with public distribution.

**Known landscape:**
| Component | License | AGPL-compatible? |
|-----------|---------|-----------------|
| whisper.cpp (engine) | MIT | Yes |
| OpenAI Whisper models | MIT-like (cc0/MIT) | Yes — verify per variant |
| Vosk API (engine) | Apache 2.0 | Yes |
| Vosk models | Apache 2.0 (most) | Yes — verify per model |
| Coqui STT engine | MPL 2.0 | Yes (weak copyleft, compatible) |
| Coqui TTS models | Various — some CC-NC | **No** — CC non-commercial incompatible |
| Silero VAD | MIT | Yes |
| OpenWakeWord | Apache 2.0 | Yes |

**Warning signs (detect early):**
- A model download URL points to a `.gguf` with no accompanying license file
- Using a "fine-tuned" Whisper variant from HuggingFace without checking the model card
- A dependency is a Python wheel that wraps a binary without clear license disclosure

**Prevention:**
- Maintain a `LICENSES.md` or SBOM from the first dependency decision
- Audit model licenses separately from inference engine licenses — they are different artifacts
- For bundled or downloaded models, check the original model card on HuggingFace or the official project site
- Default to the OpenAI upstream Whisper models (MIT) or confirmed Apache Vosk models
- Add a CI check that fails if new dependencies are added without a license entry

**Phase/component to address:**
Dependency selection phase; legal review checkpoint before first public release.

---

## Medium Pitfalls

---

### 🟡 P-13 — Whisper Model Download on First Launch (Bad Cold-Start UX)

**What goes wrong:**
If the application downloads the speech model on first run without warning, users see a progress bar downloading 75MB–1.5GB before anything works. On slow connections this takes minutes. The app appears broken.

**Prevention:**
- Bundle the smallest viable model (`whisper-tiny.en` at ~75MB) with the package, or
- Show a setup wizard on first launch that explains the download, estimates time, and allows model selection
- Never silently block on a model download during startup; make it an explicit step

**Phase/component to address:**
Distribution / first-run experience.

---

### 🟡 P-14 — ydotool / wtype Daemon Dependency (Running Services the User Didn't Opt Into)

**What goes wrong:**
Tools like `ydotool` require a background daemon (`ydotoold`) to be running. If this daemon is not started, all injection silently fails. Running it as a system-level root service causes ghost input events and instability on some compositors (reported on Ubuntu 24.04).

**Prevention:**
Do not use `ydotool` or `wtype` as the injection mechanism. Use the **`uinput` kernel interface directly** — it requires no daemon, no external process, and no root (only the correct group membership). This eliminates a whole class of daemon lifecycle problems.

**Phase/component to address:**
Input injection backend selection.

---

### 🟡 P-15 — Compositor-Specific Behavior Divergence

**What goes wrong:**
GNOME (Mutter), KDE Plasma (KWin), Sway, Hyprland, COSMIC, and labwc each implement the Wayland protocol with different extensions and different behavior around input grabs, pointer lock, and XWayland interop. A feature that works on KDE Plasma may silently fail on Sway.

**Warning signs (detect early):**
- Push-to-talk works on GNOME/X11 fallback but not Sway pure Wayland
- Mouse capture works in testing on KDE but breaks on Hyprland

**Prevention:**
- Define a supported compositor matrix early (e.g., "supported: GNOME Wayland, KDE Plasma Wayland, Sway; best-effort: Hyprland, COSMIC")
- Test on at least GNOME and Sway in CI/CD (e.g., using a headless compositor)
- Use `uinput`-based injection (kernel-level) rather than compositor protocol-level injection — kernel input bypasses compositor-specific behavior for injection

**Phase/component to address:**
Integration testing phase; compatibility matrix documentation.

---

### 🟡 P-16 — Audio Capture Blocking the Recognition Thread

**What goes wrong:**
If audio capture and ASR inference share a thread, audio buffer overruns cause dropped audio frames while inference runs. The next utterance is silently lost because the capture buffer overflowed.

**Prevention:**
- Run audio capture in a dedicated thread/task with a ring buffer
- Run ASR inference in a separate thread consuming from the ring buffer
- Monitor buffer fill level; log warnings if > 80% full
- Use non-blocking audio capture with callback-based APIs

**Phase/component to address:**
ASR pipeline architecture.

---

### 🟡 P-17 — `uinput` Module Not Persistent Across Reboots

**What goes wrong:**
The user adds themselves to the `uinput` group and loads `modprobe uinput`, and everything works. After reboot, `/dev/uinput` is gone (module not loaded) and the app silently fails.

**Prevention:**
- Write a `/etc/modules-load.d/uinput.conf` file during installation containing just `uinput`
- Package post-install hook must do this — do not rely on users doing it manually
- Startup check: if `/dev/uinput` doesn't exist, print an actionable error that includes the exact fix command

**Phase/component to address:**
Packaging / install phase.

---

## Phase-Specific Warning Matrix

| Phase / Feature | Likely Pitfall | Severity | Mitigation |
|----------------|---------------|----------|------------|
| Input backend selection | Choosing ydotool over uinput direct | 🟡 | Use uinput directly (P-14) |
| PTT / wake word design | Global hotkey deaf in-game | 🔴 | evdev grab on physical device (P-01) |
| Audio capture subsystem | PipeWire/PA format mismatch, Bluetooth mic | 🟠 | CPAL abstraction (P-11) |
| ASR engine selection | Open vocabulary on gaming jargon | 🟠 | Constrained grammar (P-06) |
| ASR pipeline threading | Buffer overrun drops utterances | 🟡 | Dedicated capture thread (P-16) |
| VAD tuning | Game noise false triggers | 🟠 | Noise gate + configurable threshold (P-07) |
| Latency profiling | Pipeline compounds to >1s | 🔴 | Measure every stage, budget 300ms (P-05) |
| Macro execution engine | Too-fast injection drops inputs | 🟠 | Configurable dwell time (P-10) |
| uinput virtual device setup | EV_REP double-fire | 🟠 | Omit EV_REP capability (P-09) |
| uinput device naming | Anti-cheat flags virtual device | 🟠 | Neutral name, user-level only (P-08) |
| Game compatibility testing | nProtect update breaks injection | 🟠 | Monitor + maintain tested-versions list (P-08) |
| Distribution / packaging | uinput inaccessible in Flatpak | 🔴 | Use .deb/.rpm + AppImage instead (P-03) |
| Distribution / packaging | Module not persistent after reboot | 🟡 | modules-load.d in post-install (P-17) |
| First-run UX | Silent failure from missing udev/group | 🔴 | Startup check with actionable error (P-02) |
| Model download | 1.5GB surprise on first launch | 🟡 | Bundle tiny model or setup wizard (P-13) |
| Dependency licensing | ASR model with non-commercial clause | 🟠 | License audit per model (P-12) |
| Compositor compatibility | Focus race minimizes game | 🔴 | Headless daemon + gamescope (P-04) |
| Compositor compatibility | Behavior diverges across compositors | 🟡 | Supported matrix + CI testing (P-15) |

---

## Sources

- [Wayland input injection security model — ydotool issues](https://github.com/ReimuNotMoe/ydotool/issues/285) — HIGH confidence
- [Flatpak uinput access limitations](https://github.com/flatpak/flatpak/issues/4137) and [security discussion](https://github.com/flatpak/flatpak/issues/5459) — HIGH confidence
- [XWayland fullscreen focus race — COSMIC compositor](https://github.com/pop-os/cosmic-comp/issues/1874) — HIGH confidence
- [XWayland keyboard focus hijack — Sway](https://github.com/swaywm/sway/pull/8699) — HIGH confidence
- [EV_REP double repeat — evsieve issue](https://github.com/KarsMulder/evsieve/issues/36) — MEDIUM confidence
- [uinput permissions guide — Fedora Discussion](https://discussion.fedoraproject.org/t/implications-of-change-permissions-on-uinput/128865/5) — HIGH confidence
- [Vosk single-threaded, noise sensitivity, gaming study](https://www.preprints.org/manuscript/202505.0855/v1) — MEDIUM confidence (single study)
- [whisper.cpp streaming + VAD](https://github.com/ggml-org/whisper.cpp/blob/master/examples/stream/README.md) — HIGH confidence
- [Helldivers 2 — nProtect GameGuard Linux/Proton](https://bbs.archlinux.org/viewtopic.php?pid=2230490) — MEDIUM confidence (community reports)
- [PipeWire Flatpak Bluetooth mic portal requirement](https://huzi.pk/blog/tech/pipewire-flatpak-bluetooth-mic-permissions) — HIGH confidence
- [LinVAM (prior art: Linux voice macro)](https://github.com/stele95/LinVAM) — HIGH confidence (existing project with same pitfalls)
- [VoxInput (prior art: voice → uinput)](https://github.com/richiejp/VoxInput) — HIGH confidence
- [AntiMicroX uinput wiki (manual udev workaround)](https://github.com/AntiMicroX/antimicrox/wiki/Open-uinput-error) — HIGH confidence
- [uinput kernel documentation](https://www.kernel.org/doc/html/v4.16/input/uinput.html) — HIGH confidence
