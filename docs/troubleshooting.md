# Troubleshooting

Quick-reference for common `vibe-attack` failure modes. Each section follows the
pattern: **symptom → likely cause → fix**.

For full uinput/permissions setup, see [docs/uinput-setup.md](uinput-setup.md) — this
guide links there rather than duplicating it.

---

## uinput / /dev/uinput

**Symptom:** `Permission denied` or `No such file or directory` on `/dev/uinput` at startup.

**Likely cause:** The `uinput` kernel module is not loaded, or your user is not in the
`input` group.

**Fix:**
```bash
# Load the module now
sudo modprobe uinput

# Add yourself to the input group (log out and back in to take effect)
sudo usermod -aG input "$USER"
```

Persist the module across reboots:
```bash
echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf
```

> **systemd v258+ (Arch / CachyOS 2025+):** Use the `input` group, not `uinput`. The
> `uinput` group was broken in systemd v258 because non-system groups are no longer
> recognized by udev rules. See [docs/uinput-setup.md](uinput-setup.md) for the full
> module-load and group setup steps.

---

## Audio / CPAL

**Symptom:** `No input devices found` or `failed to build stream` at startup.

**Likely cause:** The CPAL audio backend cannot see any recording devices, or the
configured device name is wrong.

**Fix:** List available devices, then update `config.yaml`:
```bash
vibe-attack --list-devices
```

The output shows CPAL device names. Copy the exact string for your microphone into
`~/.config/vibe-attack/config.yaml`:
```yaml
audio:
  device: "Your Device Name Here"
```

Leave `device` unset to use the system default input device. If no devices appear at
all, verify ALSA is working: `arecord -l`.

---

## Models

**Symptom:** `Model file not found` or an ONNX Runtime error on startup with STT or
wake-word enabled.

**Likely cause:** Model files must be downloaded manually — they are not bundled with
the binary.

**Fix for Whisper STT:**
```bash
mkdir -p ~/.local/share/vibe-attack/models/whisper
curl -L -o ~/.local/share/vibe-attack/models/whisper/ggml-tiny.en.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin
```

Then ensure `config.yaml` points to the file:
```yaml
stt:
  enabled: true
  model_path: ~/.local/share/vibe-attack/models/whisper/ggml-tiny.en.bin
```

**Fix for wake-word (sherpa-onnx):** Download the BPE keyword-spotting bundle from the
sherpa-onnx release page and set the `encoder`, `decoder`, `joiner`, `tokens`, and
`keywords` paths in `config.yaml`. See `config.example.yaml` for the full structure.

**ONNX Runtime errors:** `libonnxruntime.so` and `libsherpa-onnx-c-api.so` are placed
next to the binary at build time and are loaded automatically at startup. If you move
the binary to a different directory without the `.so` files, or if you see an ORT
initialization error in a custom install, set `ORT_DYLIB_PATH` to the full path of
`libonnxruntime.so`:
```bash
export ORT_DYLIB_PATH=/path/to/libonnxruntime.so
```

---

## STT Accuracy

**Symptom:** Transcripts are wrong, low-confidence, or the daemon frequently ignores
speech.

**Likely cause:** Background noise, too-short utterances, or vocabulary mismatch.

**Fixes:**
- Move to a quieter environment or use a closer microphone.
- Tune VAD thresholds in `config.yaml` — lower `start_threshold` (e.g. `0.50`) if
  speech is not being detected; raise it if noise triggers false positives.
- Provide an `initial_prompt` to bias Whisper toward known vocabulary:
  ```yaml
  stt:
    initial_prompt: "reinforce, resupply, eagle airstrike, precision strike"
  ```
- Increase `min_speech_ms` if very short accidental sounds are being transcribed.

---

## Daemon

**Symptom:** A second `vibe-attack` invocation fails, or no macros are executed even
though the daemon appears to be running.

**Likely cause:** The UDS (Unix Domain Socket) is missing or the daemon crashed silently.

**Fix:** Use the `ping` subcommand to check daemon health:
```bash
vibe-attack ping
```

A healthy daemon prints `Pong`. If the socket is stale, remove it and restart:
```bash
rm -f /run/user/"$(id -u)"/vibe-attack.sock
vibe-attack > /dev/null 2>&1 &
```

---

## Build

**Symptom:** `cargo build` fails with missing headers, linker errors, or cmake errors.

**Likely cause:** Native build dependencies are not installed.

**Fix (Debian/Ubuntu):**
```bash
sudo apt-get install libasound2-dev pkg-config cmake libclang-dev
```

**Fix (Arch/CachyOS):**
```bash
sudo pacman -S alsa-lib pkgconf cmake clang
```

`whisper-rs` builds the whisper.cpp C++ library via cmake — cmake must be on `PATH`.
If you only need PTT injection and do not need STT, build without the `stt` feature to
skip the cmake dependency:
```bash
cargo build --no-default-features
```

See `CONTRIBUTING.md` for the full list of feature flags.

**Shared library deployment:** `cargo build` automatically copies `libonnxruntime.so`
and `libsherpa-onnx-c-api.so` into the `target/` output directory alongside the binary.
Both files must be present next to the binary at runtime for wake-word detection and VAD
to work. When packaging or deploying the binary, copy those `.so` files with it.

---

## AppImage

**Symptom:** The AppImage launches but wake-word or VAD fails immediately with an ORT
initialization error, even though the AppImage was built successfully.

**Likely cause:** `libonnxruntime.so` is not being found at runtime. Inside an AppImage,
the FUSE mount path is not on the system `LD_LIBRARY_PATH`, so `dlopen` cannot locate the
bundled `.so` files unless the `AppRun` wrapper sets the path explicitly.

**How the build handles this:** `packaging/appimage/build.sh` discovers
`libonnxruntime.so` (checking `target/release/`, `ORT_DYLIB_PATH`, then the system via
`ldconfig`) and copies it into `AppDir/usr/lib/`. It then writes an `AppRun` script that
sets `LD_LIBRARY_PATH=$APPDIR/usr/lib:$LD_LIBRARY_PATH` before exec-ing the binary. As
long as you build with `build.sh`, the AppImage is self-contained.

**Manual install (without AppImage):** If you copy the binary to a custom location
without using the AppImage, ensure `libonnxruntime.so` and `libsherpa-onnx-c-api.so` are
in the same directory as the binary, or set `ORT_DYLIB_PATH`:
```bash
export ORT_DYLIB_PATH=/path/to/libonnxruntime.so
```

**Rebuilding after ORT version change:** If you upgrade `onnxruntime` on your system,
rebuild the AppImage so `build.sh` re-discovers and re-bundles the new `.so` version.
