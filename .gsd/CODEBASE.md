# Codebase Map

Generated: 2026-04-25T19:02:13Z | Files: 99 | Described: 0/99
<!-- gsd:codebase-meta {"generatedAt":"2026-04-25T19:02:13Z","fingerprint":"39c56bb56b1b18ef0d6088948fe70bc044a5b798","fileCount":99,"truncated":false} -->

### (root)/
- `.gitignore`
- `about.hbs`
- `about.toml`
- `Cargo.toml`
- `config.example.yaml`
- `config.yaml`
- `demo_hd2.yaml`
- `LICENSE`
- `LICENSES.md`
- `README.md`

### docs/
- `docs/latency-baseline.md`
- `docs/uinput-setup.md`

### docs/latency-proofs/phase-02-target-hardware/
- `docs/latency-proofs/phase-02-target-hardware/error.txt`
- `docs/latency-proofs/phase-02-target-hardware/README.md`
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

### profiles/hd2/
- `profiles/hd2/pack.yaml`

### src/
- `src/config.rs`
- `src/error.rs`
- `src/lib.rs`
- `src/main.rs`

### src/audio/
- `src/audio/mod.rs`

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

### src/vad/
- `src/vad/mod.rs`

### src/wake/
- `src/wake/mod.rs`

### tests/
- `tests/concurrency_stress.rs`
- `tests/config_parse.rs`
- `tests/daemon_headless.rs`
- `tests/dispatcher_logic.rs`
- `tests/drop_oldest_queue.rs`
- `tests/jsonl_schema.rs`
- `tests/macro_inject.rs`
- `tests/stt_smoke.rs`
- `tests/uinput_smoke.rs`
- `tests/wake_word.rs`
