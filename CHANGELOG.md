# Changelog

All notable changes to vibe-attack are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

---

## [1.1.0] - 2026-06-07

### Changed
- **VAD onset algorithm**: replaced consecutive-frame counter with N-of-M sliding-window majority vote (3 of 5 frames by default). A single low-confidence Silero frame during speech onset no longer resets the detector, significantly improving recognition of spoken commands.
- **VAD defaults tuned**: `start_threshold` 0.60→0.50, `stop_threshold` 0.45→0.30, `end_silence_ms` 200→500ms. New `onset_window_ms: 100` config field controls the window size.
- **Wake mode VAD**: removed the aggressive `stop_threshold = start - 0.05` formula; uses base stop_threshold for consistency.

### Added
- **Whisper initial_prompt**: daemon now auto-builds a comma-separated phrase list from the active pack's macros and passes it to Whisper as `initial_prompt` at startup. This biases the model toward known vocabulary, reducing hallucinations on short HD2 stratagem names. Explicit `stt.initial_prompt` in config overrides the auto-built value.
- **Sound feedback UI**: pack editor SOUND row with Browse/Clear buttons (rfd file dialog). Per-macro sound file paths now round-trip through the UI and persist to pack.yaml.
- **VAD debug tracing**: per-frame Silero scores logged at DEBUG level (`RUST_LOG=vibe_attack=debug`).
- **DEVICES nav icon**: replaced unrenderable ⊞ (U+229E, not in egui's bundled font) with 🖥 emoji.

---

## [1.0.0] - 2026-04-28

### Added
- **AppImage packaging**: `packaging/appimage/build.sh` now discovers `libonnxruntime.so`
  via a four-stage fallback (`target/release/` → `ORT_DYLIB_PATH` env var → `ldconfig`
  → Arch package path), bundles it into `AppDir/usr/lib/`, and writes an `AppRun` wrapper
  that sets `LD_LIBRARY_PATH` at runtime so ORT loads correctly inside the FUSE mount.
  `linuxdeploy`/`appimagetool` invocation is gated on `command -v` so the script
  degrades gracefully when those tools are absent.
- **PKGBUILD runtime dep**: `onnxruntime` added to `depends` so pacman enforces the
  shared-library requirement.
- **Packaging tests**: `tests/packaging.rs` has 5 static checks that verify `build.sh`
  bundles the `.so`, writes `AppRun` with the correct `LD_LIBRARY_PATH` line, and that
  `PKGBUILD` declares the runtime dependency — no build tools required to run.
- **Wake-word re-enabled**: `config.yaml` and `demo_hd2.yaml` now ship with
  `wake.enabled: true` following the ORT dual-instance conflict resolution.
- **Troubleshooting docs**: new AppImage section in `docs/troubleshooting.md` covering
  the ORT runtime-path issue, how `build.sh` handles it, and the manual `ORT_DYLIB_PATH`
  fallback.

### Fixed
- **ORT dual-instance heap corruption**: `sherpa-onnx` dependency switched from static
  to shared ORT linking (`default-features = false, features = ["shared"]`). Both
  `sherpa-onnx` (wake-word) and the `ort` crate (VAD) now share a single
  `libonnxruntime.so`, eliminating the ~218 duplicated ORT symbols that caused
  `std::bad_alloc` when both subsystems ran simultaneously.
- **XDG env-var test race**: six tests in `tests/pack_hd2_bundle.rs` that mutate
  `XDG_CONFIG_HOME` are now annotated with `#[serial]` (`serial_test = "3"`) to prevent
  data races under the parallel test harness.
- **`audio_probe.rs` cpal 0.15 API**: `Device::description()` replaced with
  `Device::name()`, `SampleRate` display corrected, and the `!Send` stream move-to-thread
  path removed (CPAL streams are `!Send` on ALSA/Linux).

### Changed
- **Project renamed from `hd-linux-voice` to `vibe-attack`**: binary name, crate name,
  config/data directory (`~/.config/vibe-attack/`, `~/.local/share/vibe-attack/`),
  socket path, `.desktop` file, PKGBUILD, README, docs, and all source references
  updated. Historical benchmark artifacts in `docs/latency-proofs/` retain original
  content for reproducibility.
