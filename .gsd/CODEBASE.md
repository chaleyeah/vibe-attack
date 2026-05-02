# Codebase Map

Generated: 2026-05-02T11:34:43Z | Files: 166 | Described: 0/166
<!-- gsd:codebase-meta {"generatedAt":"2026-05-02T11:34:43Z","fingerprint":"09206a802a3fb93e30599c20c94752a2935725ba","fileCount":166,"truncated":false} -->

### (root)/
- `.gitignore`
- `=`
- `about.hbs`
- `about.toml`
- `Cargo.toml`
- `CHANGELOG.md`
- `config.example.yaml`
- `config.yaml`
- `CONTRIBUTING.md`
- `demo_hd2.yaml`
- `LICENSE`
- `LICENSES.md`
- `README.md`
- `Vibe_Attack-x86_64.AppImage`
- `vibe-attack-x86_64.AppImage`

### .github/workflows/
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`

### AppDir/
- `AppDir/.DirIcon`
- `AppDir/AppRun`
- `AppDir/vibe-attack.desktop`

### AppDir/usr/bin/
- `AppDir/usr/bin/vibe-attack`
- `AppDir/usr/bin/vibe-attack-config`

### AppDir/usr/lib/
- `AppDir/usr/lib/libonnxruntime.so`
- `AppDir/usr/lib/libsherpa-onnx-c-api.so`

### AppDir/usr/share/applications/
- `AppDir/usr/share/applications/vibe-attack.desktop`

### docs/
- `docs/configuration.md`
- `docs/latency-baseline.md`
- `docs/troubleshooting.md`
- `docs/uinput-setup.md`

### docs/distribution-proofs/appimage/
- `docs/distribution-proofs/appimage/README.md`

### docs/distribution-proofs/appimage/cachyos/
- `docs/distribution-proofs/appimage/cachyos/transcript.md`

### docs/distribution-proofs/appimage/debian13/
- `docs/distribution-proofs/appimage/debian13/transcript.md`

### docs/distribution-proofs/appimage/fedora44/
- `docs/distribution-proofs/appimage/fedora44/transcript.md`

### docs/distribution-proofs/appimage/ubuntu2604/
- `docs/distribution-proofs/appimage/ubuntu2604/transcript.md`

### docs/distribution-proofs/aur/
- `docs/distribution-proofs/aur/README.md`

### docs/distribution-proofs/final/
- `docs/distribution-proofs/final/README.md`

### docs/distribution-proofs/final/cachyos/
- `docs/distribution-proofs/final/cachyos/transcript.md`

### docs/distribution-proofs/final/debian13/
- `docs/distribution-proofs/final/debian13/transcript.md`

### docs/distribution-proofs/final/fedora44/
- `docs/distribution-proofs/final/fedora44/transcript.md`

### docs/distribution-proofs/final/ubuntu2604/
- `docs/distribution-proofs/final/ubuntu2604/transcript.md`

### docs/distribution-proofs/wizard/
- `docs/distribution-proofs/wizard/README.md`

### docs/distribution-proofs/wizard/cachyos/
- `docs/distribution-proofs/wizard/cachyos/transcript.md`

### docs/distribution-proofs/wizard/debian13/
- `docs/distribution-proofs/wizard/debian13/transcript.md`

### docs/distribution-proofs/wizard/fedora44/
- `docs/distribution-proofs/wizard/fedora44/transcript.md`

### docs/distribution-proofs/wizard/ubuntu2604/
- `docs/distribution-proofs/wizard/ubuntu2604/transcript.md`

### docs/latency-proofs/phase-02-target-hardware/
- `docs/latency-proofs/phase-02-target-hardware/error.txt`
- `docs/latency-proofs/phase-02-target-hardware/README.md`
- `docs/latency-proofs/phase-02-target-hardware/RESULTS.md`
- `docs/latency-proofs/phase-02-target-hardware/RESULTS.template.md`
- `docs/latency-proofs/phase-02-target-hardware/timing.log`
- `docs/latency-proofs/phase-02-target-hardware/transcript.jsonl`

### docs/latency-proofs/phase-02-target-hardware/results-accelerated/
- `docs/latency-proofs/phase-02-target-hardware/results-accelerated/transcript.jsonl`

### docs/latency-proofs/phase-02-target-hardware/results-ptt/
- `docs/latency-proofs/phase-02-target-hardware/results-ptt/RESULTS.md`
- `docs/latency-proofs/phase-02-target-hardware/results-ptt/timing.log`
- `docs/latency-proofs/phase-02-target-hardware/results-ptt/transcript.jsonl`

### docs/latency-proofs/phase-02-target-hardware/results-wake/
- `docs/latency-proofs/phase-02-target-hardware/results-wake/RESULTS.md`
- `docs/latency-proofs/phase-02-target-hardware/results-wake/timing.log`
- `docs/latency-proofs/phase-02-target-hardware/results-wake/transcript.jsonl`

### examples/
- `examples/audio_probe.rs`

### models/sherpa/
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01.tar.bz2`

### models/sherpa/kws/
- `models/sherpa/kws/bpe.model`
- `models/sherpa/kws/decoder.onnx`
- `models/sherpa/kws/encoder.onnx`
- `models/sherpa/kws/joiner.onnx`
- `models/sherpa/kws/keywords_out.txt`
- `models/sherpa/kws/keywords.txt`
- `models/sherpa/kws/phrases_in.txt`
- `models/sherpa/kws/tokens.txt`

### models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/bpe.model`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/decoder-epoch-12-avg-2-chunk-16-left-64.int8.onnx`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/decoder-epoch-12-avg-2-chunk-16-left-64.onnx`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/encoder-epoch-12-avg-2-chunk-16-left-64.int8.onnx`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/encoder-epoch-12-avg-2-chunk-16-left-64.onnx`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/joiner-epoch-12-avg-2-chunk-16-left-64.int8.onnx`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/joiner-epoch-12-avg-2-chunk-16-left-64.onnx`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/keywords_raw.txt`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/keywords.txt`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/README.md`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/tokens.txt`

### models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/test_wavs/
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/test_wavs/0.wav`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/test_wavs/1.wav`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/test_wavs/test_keywords.txt`
- `models/sherpa/sherpa-onnx-kws-zipformer-gigaspeech/test_wavs/trans.txt`

### models/sherpa/test_wavs/
- `models/sherpa/test_wavs/en_0.wav`
- `models/sherpa/test_wavs/en_1.wav`
- `models/sherpa/test_wavs/en_trans.txt`
- `models/sherpa/test_wavs/keywords_raw.txt`
- `models/sherpa/test_wavs/keywords.txt`
- `models/sherpa/test_wavs/zh_0.wav`
- `models/sherpa/test_wavs/zh_1.wav`
- `models/sherpa/test_wavs/zh_2.wav`
- `models/sherpa/test_wavs/zh_3.wav`
- `models/sherpa/test_wavs/zh_4.wav`
- `models/sherpa/test_wavs/zh_5.wav`
- `models/sherpa/test_wavs/zh_6.wav`

### models/whisper/
- `models/whisper/ggml-tiny.en.bin`

### packaging/
- `packaging/PKGBUILD`
- `packaging/vibe-attack.spec`

### packaging/appimage/
- `packaging/appimage/build.sh`
- `packaging/appimage/vibe-attack.desktop`

### packaging/debian/
- `packaging/debian/changelog`
- `packaging/debian/control`
- `packaging/debian/copyright`
- `packaging/debian/rules`

### profiles/hd2/
- `profiles/hd2/pack.yaml`

### scripts/
- `scripts/setup.sh`
- `scripts/verify-appimage.sh`

### src/
- `src/config.rs`
- `src/error.rs`
- `src/lib.rs`
- `src/main.rs`

### src/audio/
- `src/audio/mod.rs`

### src/bin/
- `src/bin/vibe-attack-config.rs`

### src/control/
- `src/control/client.rs`
- `src/control/mod.rs`
- `src/control/protocol.rs`

### src/input/
- `src/input/inject.rs`
- `src/input/mod.rs`
- `src/input/ptt.rs`

### src/pack/
- `src/pack/manager.rs`
- `src/pack/mod.rs`

### src/pipeline/
- `src/pipeline/coordinator.rs`
- `src/pipeline/dispatcher.rs`
- `src/pipeline/jsonl.rs`
- `src/pipeline/matcher.rs`
- `src/pipeline/mod.rs`
- `src/pipeline/sound.rs`
- `src/pipeline/timing.rs`

### src/stt/
- `src/stt/mod.rs`

### src/tui/
- `src/tui/app.rs`
- `src/tui/editor.rs`
- `src/tui/mod.rs`

### src/ui/
- `src/ui/config_app.rs`
- `src/ui/first_run.rs`
- `src/ui/mod.rs`
- `src/ui/pack_editor.rs`
- `src/ui/probe.rs`
- `src/ui/tray.rs`
- `src/ui/wizard.rs`

### src/vad/
- `src/vad/mod.rs`

### src/wake/
- `src/wake/mod.rs`

### tests/
- *(24 files: 24 .rs)*

### uat/gui/
- `uat/gui/builderror.txt`
