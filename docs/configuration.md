# Configuration Reference

`hd-linux-voice` is configured via a single YAML file. This document explains every
option. The annotated example lives in `config.example.yaml` at the repo root.

---

## Config File Location

The daemon follows the XDG Base Directory specification:

```
~/.config/hd-linux-voice/config.yaml
```

If `$XDG_CONFIG_HOME` is set, it uses that instead of `~/.config`.

To use a different file, pass `--config` on the command line:
```bash
hd-linux-voice --config /path/to/my-config.yaml daemon
```

The `--config` flag is the only way to override the path at runtime — there is no
environment-variable fallback.

---

## ptt — Push-to-Talk Key

```yaml
ptt:
  key: KEY_LEFTCTRL
```

`key` is an evdev key name. While the button is held, the daemon records audio and,
when released, runs the speech pipeline.

**Finding your key name:** Run `evtest` as root, select your keyboard or device, and
press the key. The output prints the event code and name, for example `KEY_LEFTSHIFT`.

Common values: `KEY_LEFTCTRL`, `KEY_RIGHTALT`, `KEY_CAPSLOCK`, `KEY_F13`–`KEY_F24`.

---

## audio — Capture Device

```yaml
audio:
  device: ~   # unset = system default
```

When `device` is unset (or `~`), CPAL picks the system default input device. To target
a specific microphone, set `device` to the exact CPAL device name:

```yaml
audio:
  device: "CORSAIR HS80 RGB Wireless Gamin, USB Audio"
```

List available names:
```bash
hd-linux-voice --list-devices
```

---

## timing — Key Injection Timing

```yaml
timing:
  dwell_ms: 50
  gap_ms: 30
```

| Key | Default | Description |
|-----|---------|-------------|
| `dwell_ms` | `50` | Milliseconds each injected key is held down |
| `gap_ms` | `30` | Milliseconds between consecutive key events in a sequence |

Increase `dwell_ms` if a game misses short key presses. Per-key overrides are supported
in the `macros` section.

---

## pipeline — Processing Pipeline

```yaml
pipeline:
  verbosity: summary
  listen_window_secs: 5
```

| Key | Default | Description |
|-----|---------|-------------|
| `verbosity` | `summary` | `summary` emits one JSONL line per utterance; `stages` adds per-stage timing events on stderr |
| `listen_window_secs` | `5` | Seconds to stay in LISTENING state after a wake-word trigger |

`listen_window_secs` is only relevant when wake-word detection is enabled.

---

## vad — Voice Activity Detection

```yaml
vad:
  start_threshold: 0.60
  stop_threshold: 0.45
  min_speech_ms: 100
  end_silence_ms: 200
  preroll_ms: 150
  tail_ms: 150
  max_utterance_secs: 10
```

| Key | Default | Description |
|-----|---------|-------------|
| `start_threshold` | `0.60` | Silero VAD score above which speech starts (hysteresis high) |
| `stop_threshold` | `0.45` | Silero VAD score below which speech ends (hysteresis low) |
| `min_speech_ms` | `100` | Minimum speech duration; shorter bursts are discarded |
| `end_silence_ms` | `200` | Silence required to commit the end of an utterance |
| `preroll_ms` | `150` | Audio prepended before the detected speech start |
| `tail_ms` | `150` | Audio appended after the detected speech end |
| `max_utterance_secs` | `10` | Hard cap; longer utterances are force-flushed with a warning |

If the daemon frequently misses the start of speech, lower `start_threshold`. If it
triggers on noise, raise it.

---

## stt — Speech-to-Text

```yaml
stt:
  enabled: false
  model_path: ~/.local/share/hd-linux-voice/models/whisper/ggml-tiny.en.bin
  # initial_prompt: "reinforce, resupply, eagle airstrike"
```

| Key | Default | Description |
|-----|---------|-------------|
| `enabled` | `false` | Set to `true` to activate Whisper transcription |
| `model_path` | see below | Path to a local ggml Whisper model binary |
| `initial_prompt` | unset | Comma-separated phrases to bias Whisper toward known vocabulary |

Models are **not downloaded automatically**. See [docs/troubleshooting.md](troubleshooting.md#models)
for the one-time download command. The recommended default path is:

```
~/.local/share/hd-linux-voice/models/whisper/ggml-tiny.en.bin
```

---

## wake — Wake-Word Detection

```yaml
wake:
  enabled: false
  encoder: ~/.local/share/hd-linux-voice/models/sherpa/kws/encoder.onnx
  decoder: ~/.local/share/hd-linux-voice/models/sherpa/kws/decoder.onnx
  joiner:  ~/.local/share/hd-linux-voice/models/sherpa/kws/joiner.onnx
  tokens:  ~/.local/share/hd-linux-voice/models/sherpa/kws/tokens.txt
  keywords: ~/.local/share/hd-linux-voice/models/sherpa/kws/keywords.txt
```

When `enabled: true`, the daemon runs a sherpa-onnx keyword spotter in the background.
A successful keyword detection opens the `listen_window_secs` recording window.

All five model paths must point to a compatible BPE keyword-spotting bundle. Place
`bpe.model` in the same directory as `encoder.onnx` — it is auto-detected.

> **Note:** Running both STT and wake-word simultaneously may cause ONNX Runtime
> conflicts. If you see initialization errors, enable only one at a time.

---

## macros — Phrase-to-Key Mappings

```yaml
macros:
  - name: test_sequence
    keys:
      - key: KEY_UP
      - key: KEY_DOWN
      - key: KEY_LEFT
      - key: KEY_RIGHT
        dwell_ms: 100   # per-key override
```

Each macro entry has:

| Key | Required | Description |
|-----|----------|-------------|
| `name` | yes | The phrase Whisper must recognise to trigger this macro |
| `keys` | yes | Ordered list of evdev key names to inject |
| `keys[].key` | yes | evdev key name (e.g. `KEY_UP`) |
| `keys[].dwell_ms` | no | Per-key hold duration, overrides the global `timing.dwell_ms` |

Phrase matching is case-insensitive. If two macro names overlap (one is a prefix of
another), the longer match takes precedence.
