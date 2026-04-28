# S02: First-run wizard end-to-end UAT — Research

**Date:** 2026-04-27

## Summary

The first-run wizard is structurally complete and compiles cleanly under `--features gui`. It is a four-step sequential state machine (CreateConfig → InstallModel → SetupUinput → ConfigurePtt) implemented in `src/ui/wizard.rs` (813 LOC) and driven by `src/ui/first_run.rs` (`FirstRunState` + `SetupStep`). The binary entry point is the single `src/bin/vibe-attack-config.rs` — there is no separate `vibe-attack` launcher; the config app IS the entry point for the wizard. All 16 `tests/ui_distribution.rs` tests pass. `probe.rs` already has hermetic serial tests for each of the four checks (config, model, uinput, ptt) using XDG env-var isolation.

What is missing for S02: (1) there are no UAT transcripts under `docs/distribution-proofs/wizard/` (the directory does not exist); (2) the `--skip-wizard` CLI flag specified in the M010 roadmap is not implemented anywhere — the binary has no CLI arg parsing at all; (3) the `.desktop` file `Exec=vibe-attack` points to a binary that does not exist (the binary is `vibe-attack-config`); (4) there is no automated test that exercises the wizard → completion → main-config transition edge (the `setup_just_completed` flag path in the binary is untested); (5) `rewrite_ptt_key` unit tests are `#[cfg(feature = "gui")]`-gated, so they only run in a GUI build — the planner should note they are tested but require `--features gui` to surface.

The primary risk for UAT on all three distros is uinput: on Wayland-only systems (Fedora 40+, CachyOS) `pkexec` may fail to display a polkit dialog with no X11/XWayland fallback. The HuggingFace model download is the second risk — network proxies, corporate firewalls, and rate limits can cause the download to fail silently (the wizard shows "Download failed: …" but there is no retry-with-local-file path). These two risks are the highest-priority items to document in UAT scenarios.

## Recommendation

Structure UAT around three distros (Debian, Fedora, Arch), three scenarios per distro: (A) clean fresh install — all four wizard steps exercised end-to-end; (B) partial state — model already placed at XDG data path before launch, wizard should skip step 2; (C) relaunch after wizard completes — wizard is not shown, main config screen appears. For the `--skip-wizard` flag: implement it as a simple `std::env::args().any(|a| a == "--skip-wizard")` check in `main()` before `probe::run()`, which substitutes `FirstRunState::from_checks(true, true, true, true)`. This is the minimal implementation described in the roadmap. The `.desktop` `Exec=vibe-attack` must be corrected to `vibe-attack-config` or a wrapper script added.

## Implementation Landscape

### Key Files

- `src/ui/wizard.rs` — 813-line wizard implementing the full state machine behind `#[cfg(feature = "gui")] mod inner`. Entry point is `show_wizard(ui, state, ptt, dl, uinput, config_example_contents, hd2_profile_contents)`. Each step is a private `show_*` function. Background threads for download, PTT capture, modprobe, and usermod are joined each frame via handle polling. The `rewrite_ptt_key` pure function is separately unit-tested (3 tests, GUI-feature-gated). `download_model` uses `ureq` with atomic rename for crash safety.

- `src/ui/first_run.rs` — 115-line pure-logic module. `SetupStep` enum (4 variants in wizard order). `FirstRunState::from_checks(config_exists, model_installed, uinput_accessible, ptt_configured)`. Methods: `is_setup_complete()`, `steps_remaining() -> Vec<SetupStep>`, `first_incomplete_step() -> Option<SetupStep>`. Five unit tests cover all step combinations; no egui dependency, compiles without `gui` feature.

- `src/ui/probe.rs` — 248-line environment probe. `run() -> FirstRunState` calls `check_config()` (XDG config path file exists), `check_model()` (XDG data path non-empty file), `check_uinput()` (open `/dev/uinput` read+write), `check_ptt()` (config file contains `key: KEY_*` line). All four checks have hermetic serial tests using XDG env-var redirection to tempdirs. Config path: `~/.config/vibe-attack/config.yaml`. Model path: `~/.local/share/vibe-attack/models/whisper/ggml-tiny.en.bin`.

- `src/bin/vibe-attack-config.rs` — 537-line binary. `VibeAttackConfigApp` holds `first_run: FirstRunState`, `ptt`, `dl`, `uinput`, plus `setup_just_completed: bool` flag. The `ui()` method dispatches to `show_wizard()` while `!first_run.is_setup_complete()` and to `show_main_config()` after. Wizard completion detection: `was_incomplete && self.first_run.is_setup_complete()` sets `setup_just_completed = true`; the next frame loads profiles and spawns mic thread. No CLI arg parsing exists; `--skip-wizard` is NOT implemented.

- `tests/ui_distribution.rs` — 166 lines, 16 tests. All pass with default features. Tests cover: `FirstRunState` step logic, step ordering, `ConfigApp` profile count and log-capping, `packaging/PKGBUILD` field presence, `.desktop` key presence, `build.sh` existence and shebang, and `[features] default` excluding `gui`. No tests exist for the wizard → completion transition or for the `--skip-wizard` path.

- `packaging/appimage/vibe-attack.desktop` — `Exec=vibe-attack` (INCORRECT — binary is `vibe-attack-config`; this must be fixed before AppImage UAT).

- `docs/distribution-proofs/wizard/` — directory does NOT exist; must be created and populated with three UAT transcripts (one per distro) after manual runs.

### Build Order

1. **Fix `.desktop` Exec target** — change `Exec=vibe-attack` to `Exec=vibe-attack-config`. Unblocks AppImage UAT on all distros.
2. **Implement `--skip-wizard` CLI flag** in `src/bin/vibe-attack-config.rs` `main()`. Required by roadmap; needed for the "relaunch skips wizard" UAT scenario. Minimal: `std::env::args().any(|a| a == "--skip-wizard")` before probe, substituting `from_checks(true,true,true,true)`.
3. **Create `docs/distribution-proofs/wizard/` directory** and UAT transcript template.
4. **Add integration test for wizard → completion transition** in `tests/ui_distribution.rs`: verify `setup_just_completed` logic by constructing a `FirstRunState` that transitions from incomplete to complete and asserting it triggers the profile/mic load path. This is a unit-level test of the state machine, not an egui test.
5. **Run manual UAT on Debian, Fedora, Arch** following the three-scenario script; produce transcript files.

### Verification Approach

**Automated (CI-safe):**
- `cargo test --test ui_distribution` — currently 16 pass; add the wizard-completion transition test here.
- `cargo test --lib -- first_run` (no features) — 5 tests pass.
- `cargo test --lib -- probe` (with `--test-threads=1`) — 8 serial tests.
- `cargo test --features gui --lib -- wizard` — 3 `rewrite_ptt_key` tests.

**Manual UAT (per distro — produce transcript):**
- Scenario A (fresh install): `rm -rf ~/.config/vibe-attack ~/.local/share/vibe-attack && vibe-attack-config` → wizard shows step 1; click "Copy example config"; step 2 appears; download model OR place stub file; step 3: modprobe + usermod (or confirm already accessible); step 4: press PTT key; wizard clears; main config screen appears.
- Scenario B (partial state): place stub model at `~/.local/share/vibe-attack/models/whisper/ggml-tiny.en.bin` before launch; wizard shows step 1 only, then jumps past step 2.
- Scenario C (relaunch): after scenario A completion, re-run `vibe-attack-config`; wizard must NOT appear; main config screen shown immediately; PTT key and profiles are loaded.
- Scenario D (skip-wizard flag): `vibe-attack-config --skip-wizard`; main config screen appears without wizard regardless of disk state.

**Transcript format** (suggested for `docs/distribution-proofs/wizard/`):
```
# Wizard UAT — <Distro> <Version>
Date: YYYY-MM-DD
Tester: <name>
Binary: vibe-attack-config <version>

## Scenario A: Fresh Install
[step-by-step observation + PASS/FAIL per step]

## Scenario B: Partial State (model pre-placed)
...

## Scenario C: Relaunch (wizard skipped)
...
```

## Constraints

- All wizard egui rendering is gated behind `#[cfg(feature = "gui")]`; tests of pure logic (FirstRunState, rewrite_ptt_key, probe checks) compile without it but GUI integration requires `--features gui` build.
- The PTT capture thread uses `evdev::enumerate()` which reads `/dev/input/event*`; requires the user to be in the `input` group (same group required for uinput). On a freshly set-up machine neither group membership may exist.
- `pkexec` (polkit) for modprobe/usermod requires a running polkit agent. On headless CI and minimal Wayland desktops without XDG_SESSION_TYPE=x11 or a polkit agent running, pkexec blocks or fails. UAT testers must run a full desktop session.
- The `uinput` probe opens `/dev/uinput` read+write; on systems where the `uinput` module is not loaded, this fails with ENOENT even if the user is in the `input` group. Both steps (modprobe + usermod) must succeed and a re-login or `newgrp input` must be done.
- `ureq` (used for model download) does not follow HTTP redirects by default in all versions; HuggingFace uses CDN redirects. Should verify download completes successfully on the actual distros.
- `cargo test` must use `--test-threads=1` for probe tests due to XDG env-var races (MEM074, MEM008).

## Common Pitfalls

- **`.desktop` Exec target is wrong** — `Exec=vibe-attack` will fail because only `vibe-attack-config` exists as a binary; fix before any AppImage packaging run.
- **Manual key entry in PTT step does not re-probe** — if user types a key name manually and presses Enter, `probe::run()` is called and state updates, but the `ptt.listening` flag is not cleared for the manual path. Review `show_configure_ptt`: the manual entry path correctly calls `*state = probe::run()` but does not reset `ptt.listening` (it was never set, so this is fine). Verify on real hardware.
- **evdev PTT capture picks wrong device** — `find_keyboard_device()` returns the FIRST device supporting KEY_A; on machines with multiple keyboards (gaming keyboards, virtual devices, HID dongles), it may not capture from the user's intended device. If capture hangs, user must use manual entry fallback.
- **Wayland polkit dialog may not appear** — on Fedora/GNOME Wayland sessions, pkexec requires `xhost +SI:localuser:root` or a polkit agent. Testers should confirm polkit agent is running (`ps aux | grep polkit`).
- **HuggingFace download may 302-redirect** — test download manually with `curl -L <MODEL_URL>` before UAT to confirm ureq handles the redirect chain.
- **`setup_just_completed` transition** — the boolean is only set in the frame where `was_incomplete && is_setup_complete()` — if the first render frame already starts complete (e.g. after wizard), this path is skipped. Since probe runs at construction time and the wizard sets state synchronously, this should be fine for normal paths, but verify the Scenario C (relaunch) case to confirm `setup_just_completed` is never spuriously set to true when starting complete.

## Open Risks

- **Wayland compositor PTT capture** — evdev capture works when user is in the `input` group regardless of compositor, but polkit pkexec dialogs may not display on pure Wayland without a configured polkit agent. Risk: moderate on Fedora 40+/GNOME Wayland.
- **`input` group vs session-time membership** — `usermod -aG input $USER` does not take effect in the current login session; user must `newgrp input` or re-login. The wizard shows this instruction but does NOT re-probe automatically after `newgrp` (cannot — that requires a new shell). Users may be confused by persistent "uinput not accessible" state after clicking "Re-check" in the same session. Mitigation: add wording "log out and back in, then relaunch".
- **HuggingFace network access** — model download from `huggingface.co` may be blocked by corporate/educational proxies. Fallback (manual local file path) is not wired up in the wizard UI. Risk: high in restricted-network environments. Mitigation for UAT: ensure testers have direct internet; document manual placement path.
- **Model URL stability** — the hardcoded URL `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin` is in `wizard.rs` as a constant. If HuggingFace moves the file, the download silently fails. No integrity check (checksum) is performed.
- **AppImage sandboxing of `/dev/input`** — if the AppImage is built with FUSE-level sandboxing or runs under Flatpak security policy, evdev enumeration may be blocked. Verify `vibe-attack-config.desktop` does not include `Sandbox=` directives and the AppImage does not wrap in bwrap by default.
- **No automated end-to-end GUI test** — there is no headless egui test (no `eframe` test harness); all wizard panel logic can only be verified manually. The pure-logic layer (FirstRunState, probe, rewrite_ptt_key) is tested, but the rendering and button-click dispatch are not. This is expected given the egui architecture; note it as a permanent constraint.
