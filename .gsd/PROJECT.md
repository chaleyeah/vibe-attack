# hd-linux-voice

## What This Is

An **open source** Linux desktop application in the spirit of [VoiceAttack](https://voiceattack.com/) that binds spoken phrases to keyboard and mouse macros so players can trigger in-game actions by voice. The first concrete target is **Helldivers 2**: ship a maintained set of **stratagem** voice commands that map to the correct key sequences, usable during live gameplay. The project is licensed under **GNU AGPL v3.0** (see `LICENSE`); distribution is aimed at a **small release** (other users installing and running it), not a single-developer-only script.

## Core Value

**During Helldivers 2 gameplay, the player can fire the right stratagem reliably by voice with minimal delay and without breaking flow** — wake word or push-to-talk, fully **local** speech processing, **Wayland-first** input delivery.

## Current State (2026-06-07)

**M014 v1.1 — complete ✅**

| Milestone | Status | Delivered |
|-----------|--------|-----------|
| M001: Migration | ✅ complete | Rust toolchain, pipeline (VAD/STT/wake), phrase dispatch, pack system, UI scaffolding, docs, dual-ORT fix |
| M007: Codebase Cleanup & Documentation | ✅ complete | Dead code removal, load_profiles fix, 191 pub items documented, 10 doc drift items corrected |
| M008: UI / Tray Runtime Control | ✅ complete | Control protocol extensions, egui config window, tray Mode submenu, headless integration tests |
| M009: Pack UX — Editor, Import/Export, Full HD2 Coverage | ✅ complete | Full pack editor UI (CRUD, import/export, Test button); 75-stratagem HD2 pack; hermetic coverage tests |
| M010: Distribution — AppImage, AUR, First-Run Wizard | ✅ complete | AppImage CI pipeline, AUR PKGBUILD, first-run wizard, proof harness, README rewrite |
| M011: v1.0 Release | ✅ complete | Retargeted proof harness to 4 current distros; 5 UI/UX bug fixes; 4-job release CI; GitHub Release v1.0.0 live with AppImage + .deb + .rpm + tarball + hdpack; PKGBUILD sha256sums pinned |
| M012: GUI Redesign | ✅ complete | Tactical dark-panel egui theme + widgets; all 5 config panes, pack editor, wizard |
| M013: CI Build Revamp | ✅ complete | Release pipeline fixes, RPM/deb version from git tag, CI badges |
| M014: v1.1 STT + VAD + Sound UI | ✅ complete | N-of-M VAD onset, tuned defaults, Whisper initial_prompt from pack phrases, sound feedback UI, DEVICES icon fix |

### M011 Slice Detail

| Slice | Status | Delivered |
|-------|--------|-----------|
| S01: Rename proof directories and update test harness | ✅ complete | 16 tests retargeted to debian13, ubuntu2604, fedora44, cachyos; old distro dirs removed |
| S02: VM proof runs — populate transcripts | ✅ complete | ubuntu2604 AppImage STATUS: ok (real run); all 12 transcripts structurally valid |
| S03: UI polish from proof-run findings | ✅ complete | 5 bugs fixed (wizard PTT state, download auto-advance, tray Quit, mode tooltip, config weak text); 105 tests pass |
| S04: Version bump + release CI | ✅ complete | 1.0.0 across all manifests; CHANGELOG dated; 4-job release.yml; 15 packaging tests |
| S05: Publish GitHub Release v1.0.0 | ✅ complete | GitHub Release v1.0.0 live; 5 artifacts uploaded; PKGBUILD sha256sums pinned |

## Requirements

### Validated

- ACT-01: Push-to-talk via evdev — Validated in M001/S01 (Phase 1 Foundation)
- MCRO-01: Key sequences with configurable inter-key delays — Validated in M001/S01 (Phase 1 Foundation)
- MCRO-02: Key/button holds (press-and-hold dwell) — Validated in M001/S01 (Phase 1 Foundation)
- MCRO-05: uinput/evdev key events on Wayland — Validated in M001/S01 (Phase 1 Foundation)
- UI-01: Headless daemon mode — Validated in M001/S01 (Phase 1 Foundation)
- DIST-03: AGPL-3.0 license + LICENSES.md inventory — Validated in M001/S01 (Phase 1 Foundation)
- ACT-03: PTT ↔ wake-word mode switch from config UI — Validated in M008 (SetMode via config window + tray; coordinator drain; no pipeline restart)
- ACT-04: Tray icon reflects listening state — Validated in M008 (icon_name_for_state; all DaemonState variants; 5 unit tests)
- STT-02: Confidence threshold fuzzy matching — Validated in M008 (RwLock<PhraseMatcher>; live update_threshold(); test passes)
- STT-03: Configure threshold from config UI — Validated in M008 (threshold_pct slider; SetThreshold dispatch; integration test passes)
- UI-02: System tray icon with status and controls — Validated in M008 (tray icon + Mode submenu + profile submenu complete)
- UI-03: Config window for audio/mode/threshold/keybindings — Validated in M008 (egui config window; atomic YAML save; daemon-absent recovery)
- PACK-05: Multiple named profiles, runtime-switchable — Validated in M008/M009 (SwitchProfile protocol, ProfileManager, tray submenu; runtime switching confirmed)
- MCRO-04: Sound feedback on macro activation — Validated in M014 (SoundPlayer + Dispatcher wiring; per-macro sound field in pack editor with Browse/Clear UI)

### Advanced (structural foundation complete, runtime validation pending)

- PACK-01: HD2 pack bundle — 75 macros across 6 categories; hermetic coverage test guards; runtime CI confirmation pending
- PACK-02: Import packs from .hdpack — implementation complete (Pack::import_to + egui Import button + hermetic tests); formal validation pending
- PACK-03: Export packs to .hdpack — implementation complete (Pack::export + egui Export button + hermetic tests); formal validation pending
- PACK-04: Built-in macro editor — implementation complete (PackEditor + PackEditorState + egui panel + CRUD tests); formal validation pending
- UI-04: First-run wizard — --skip-wizard flag wired; wizard_proofs.rs passes; VM end-to-end runs pending
- DIST-01: AppImage — GitHub Release v1.0.0 live; ubuntu2604 AppImage STATUS: ok; debian13/fedora44/cachyos VM runs pending
- DIST-02: AUR/PKGBUILD — PKGBUILD sha256sums pinned; AUR git push submission pending operator runbook

### Active

- [ ] MCRO-03: Macro engine supports **conditional logic** (if/else, variables) for VoiceAttack-class scripting

### Out of Scope

- **Windows / macOS** clients in v1 (Linux-only focus; other OS may be future work).
- **Cloud-hosted** speech recognition as a **required** path for core play (optional pluggable backends may be considered later; v1 core path is local-only per decision).

## Post-v1.0 Operator Runbook

1. VM runs: follow docs/distribution-proofs/appimage/\*/transcript.md Reproduction Notes for debian13, fedora44, cachyos; update STATUS fields
2. VM runs: follow docs/distribution-proofs/wizard/\*/transcript.md Reproduction Notes for all four distros; update STATUS fields
3. AUR submission: run makepkg, namcap, clean-chroot build; `git push aur` per docs/distribution-proofs/aur/README.md
4. Make repository public to enable unauthenticated release asset downloads
5. Replace placeholder assets/vibe-attack.png with a real SVG/PNG icon

## Context

- **Inspiration:** VoiceAttack on Windows — phrase recognition, macro scripting, profiles. This project targets **feature depth over time**, not a one-off script.
- **Game focus:** Helldivers 2 stratagems are the primary driver for v1 content and for proving latency, accuracy, and input correctness under pressure.
- **Platform:** Primary display server is **Wayland**; keyboard/mouse injection and focus behavior must be validated on target distros. **X11** may follow; design should isolate input backends.
- **Implementation language:** **Rust** — performance, single-binary deployment, strong systems fit for audio + input, AGPL-compatible ecosystem.
- **Licensing:** Project is **AGPL-3.0**; third-party libraries must be compatible with AGPL distribution.
- **Risks:** Game **anti-cheat / input policies**, exclusive fullscreen vs windowed behavior, audio device contention, achievable **end-to-end latency**. The dual ORT conflict (sherpa-onnx static vs ort crate dynamic) was **resolved in M001/S07** via shared ORT linking.

## Constraints

- **License**: **AGPL-3.0** — third-party speech, UI, and model dependencies must be **AGPL-compatible**.
- **Privacy / offline**: Core path **local-only** recognition; no dependency on cloud for default gameplay.
- **Display**: **Wayland-first** implementation and testing.
- **Distribution**: "Small release" — installer expectations, sane defaults, and supportability matter from early milestones.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Local-only speech for core path | Privacy, offline play, predictable latency | Active |
| Wayland-first input | Matches stated primary environment | Active |
| Packs + built-in editor for stratagems | Bulk updates + user tweaks without blocking each other | Active |
| Open source under **AGPL-3.0** | User correction; aligns with `LICENSE` | Active |
| Long-term VoiceAttack-class depth | User ambition; phased delivery to reduce v1 risk | Active |
| Rust as implementation language | Single-binary deployment, systems-level audio+input, AGPL-compatible ecosystem | Confirmed in M001/S01 |
| Shared ORT linking for sherpa-onnx | Resolves dual-ORT bad_alloc: sherpa-onnx shared feature + ORT_DYLIB_PATH auto-discovery | Resolved in M001/S07 |
| Feature-gated GUI binary (required-features = ["gui"]) | Keeps daemon headless by default; no runtime cfg guards needed | Established in M001/S05 |
| Stdout reserved for JSONL only | Composable with tooling; tracing/stderr for all instrumentation | Established in M001/S02 |
| Dispatcher injected writer | Box<dyn Write + Send> makes dispatch testable without display server or stdout capture | Established in M001/S03 |
| Profile loading canonical format: {name}/pack.yaml subdirectories | Aligns load_profiles UI with handle_switch_profile and Pack::load_from_dir; flat *.yaml profiles no longer recognized | Established in M007/S01 |
| Every pub item in src/ requires a /// doc comment | Mechanically auditable floor for contributor comprehension; enforced by audit script + cargo doc | Established in M007 |
| Every unsafe block and #[allow(...)] requires an adjacent justifying comment | Makes invariants discoverable without git blame; one // SAFETY: per impl block | Established in M007/S02 |
| RwLock<PhraseMatcher> for live threshold updates | Threshold consumed by PhraseMatcher::new(); full matcher replacement under write lock is cleaner than two separate atomics | Established in M008/S01 |
| ActivationMode runtime-only in M008 (not persisted to YAML) | Avoids YAML schema change; SetMode sent over socket on Save; consider write-back in future milestone | Established in M008/S02 |
| Arc<RwLock<ActivationMode>> on DaemonHandle | ActivationMode not atomically encodable; matches active_profile pattern; Status always coherent with last SetMode | Established in M008/S03 |
| icon_name_for_state as free pub(crate) function | Enables unit tests without D-Bus/ksni instantiation | Established in M008/S03 |
| Tray menu activate closures use fire-and-forget spawn | ksni D-Bus callbacks must not block | Established in M008/S03 |
| pub mod pack_editor; ungated in mod.rs (wizard pattern) | Pure-logic helpers (parse_key_sequence, build_macro_config_from_form) compile under default build for unit testing | Established in M009/S03 |
| Pure-logic form helpers outside #[cfg(feature="gui")] gate | Separates testable logic from untestable egui rendering code | Established in M009/S03 |
| Rename Category does NOT cascade if_flag/set_flag references | Cascading is complex and flags are user-managed strings; surface visible warning + two-click confirm instead | Established in M009/S03 |
| Pack::import_to accepts parent profiles dir (not pack subdir) | Function appends pack.name internally; consistent with Pack::import contract | Established in M009/S04 |
| score=1.0 in Dispatcher::fire_named for control-plane triggers | Disambiguates direct control-plane fires from fuzzy phrase-matched scores in JSONL consumer | Established in M009/S05 |
| TestMacro handler uses block_in_place (multi_thread Tokio flavor required) | block_in_place cannot run on a single-thread executor; tests must declare flavor="multi_thread" | Established in M009/S05 |
| STATUS: skipped:tools-missing exit-0 in verify-appimage.sh | Validates harness structure without requiring linuxdeploy/appimagetool; partial proof inspectable via FAILURE_REASON | Established in M010/S01 |
| Pending-VM-run transcript pattern (MEM079) | Structural tests pass before VM runs; human operator converts STATUS: pending to ok at release time | Established in M010 |
| onnxruntime in PKGBUILD depends (not makedepends) | RPATH=$ORIGIN only works in AppImage; Arch native install requires system onnxruntime package at /usr/lib/ | Established in M010/S04 |
| SHERPA_ONNX_ARCHIVE_DIR=$srcdir escape hatch | Prevents sherpa-onnx-sys network downloads inside makepkg sandbox; source[] entry provides prebuilt archive | Established in M010/S04 |
| zip -j (junk-paths) for hdpack | pack.yaml lands at archive root, not nested under profiles/hd2/ | Established in M010/S03 |
| 4-job release workflow (build-appimage + build-deb + build-rpm → release) | Symmetric architecture: each build job emits upload-artifact; release job collects and publishes; independently retryable | Established in M011/S04 |
| rpmbuild --nodeps on Ubuntu CI | Ubuntu apt pre-installs all build deps; --nodeps skips Fedora-style BuildRequires resolution that would fail on apt hosts | Established in M011/S05 |
| LD_LIBRARY_PATH for linuxdeploy (dlopen-only .so) | libsherpa-onnx-c-api.so is loaded via dlopen, not ELF RPATH; ldd cannot find it; LD_LIBRARY_PATH forces dlopen search | Established in M011/S05 |
| find_so() fallback to target/sherpa-onnx-prebuilt/ | Rust build cache hits suppress cargo build --release entirely; .so never copied to target/release/; prebuilt cache is the fallback | Established in M011/S05 |
| egui PttCaptureState field (not frame-local variable) for PTT key capture | egui repaints every frame; frame-local variables reset on every repaint; stateful captures must live in a persistent struct | Established in M011/S03 |
| Arc<AtomicBool> + take pattern for tray Quit signal | Mirrors open_window pattern; avoids process::exit in ksni D-Bus callbacks which must not terminate the process abruptly | Established in M011/S03 |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):

1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):

1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-29 — M011 complete: v1.0.0 released to GitHub (AppImage + .deb + .rpm + tarball + hdpack); 4-distro proof harness; 5 UI/UX bug fixes; PKGBUILD sha256sums pinned. Pending: operator VM runs on debian13/fedora44/cachyos; AUR git push submission.*
