# S01: Control-protocol extensions — UAT

**Milestone:** M008
**Written:** 2026-04-28T01:28:35.915Z

# S01 UAT — Control Protocol Extensions

## Preconditions

- `cargo build --release` succeeds
- `$XDG_RUNTIME_DIR/vibe-attack/` directory exists (or daemon creates it on startup)
- A valid `config.yaml` is present (copy from `config.example.yaml` if needed)
- `nc` (netcat) available for Unix socket testing

---

## Test Cases

### TC-01: SetThreshold round-trip via control socket

**Steps:**
1. `cargo run --release` — start daemon, wait for startup log
2. `echo '{"cmd":"set_threshold","args":{"threshold":0.6}}' | nc -U $XDG_RUNTIME_DIR/vibe-attack/vibe-attack.sock`

**Expected:** Response is `{"status":"ok"}`. Daemon log contains `threshold_updated old=<prev> new=0.6`.

---

### TC-02: SetThreshold clamping (out-of-range value)

**Steps:**
1. Daemon running (from TC-01)
2. `echo '{"cmd":"set_threshold","args":{"threshold":1.5}}' | nc -U $XDG_RUNTIME_DIR/vibe-attack/vibe-attack.sock`

**Expected:** Response is `{"status":"ok"}`. Daemon log shows threshold clamped to `1.0` (not 1.5).

---

### TC-03: SetMode PTT→Wake

**Steps:**
1. Daemon running in default PTT mode
2. `echo '{"cmd":"set_mode","args":{"mode":"wake"}}' | nc -U $XDG_RUNTIME_DIR/vibe-attack/vibe-attack.sock`

**Expected:** Response is `{"status":"ok"}`. Daemon log contains `runtime_command_applied cmd=set_mode mode=wake`. No daemon restart occurs (pid unchanged). PTT presses no longer trigger recording; wake-word branch becomes active.

---

### TC-04: SetMode Wake→PTT

**Steps:**
1. Daemon in wake mode (from TC-03)
2. `echo '{"cmd":"set_mode","args":{"mode":"ptt"}}' | nc -U $XDG_RUNTIME_DIR/vibe-attack/vibe-attack.sock`

**Expected:** Response is `{"status":"ok"}`. Log contains `runtime_command_applied cmd=set_mode mode=ptt`. Wake-word detection stops; PTT branch becomes active.

---

### TC-05: SetMode with bogus value

**Steps:**
1. `echo '{"cmd":"set_mode","args":{"mode":"turbo"}}' | nc -U $XDG_RUNTIME_DIR/vibe-attack/vibe-attack.sock`

**Expected:** Response contains an error (non-ok status or serde deserialization failure logged). Daemon does not crash.

---

### TC-06: SetInputDevice (deferred — restart-required warning)

**Steps:**
1. `echo '{"cmd":"set_input_device","args":{"device":"hw:1,0"}}' | nc -U $XDG_RUNTIME_DIR/vibe-attack/vibe-attack.sock`

**Expected:** Response is `{"status":"ok"}`. Daemon log contains `WARN` with "restart-required" or "restart required" message. Audio device is NOT changed live (S02/S03 scope).

---

### TC-07: SetPttBinding (deferred — restart-required warning)

**Steps:**
1. `echo '{"cmd":"set_ptt_binding","args":{"key":"ctrl"}}' | nc -U $XDG_RUNTIME_DIR/vibe-attack/vibe-attack.sock`

**Expected:** Response is `{"status":"ok"}`. Daemon log contains `WARN` with restart-required message. PTT binding is NOT changed live.

---

### TC-08: ReloadConfig

**Steps:**
1. Edit `config.yaml` to set `stt.confidence_threshold: 0.75`
2. `echo '{"cmd":"reload_config"}' | nc -U $XDG_RUNTIME_DIR/vibe-attack/vibe-attack.sock`

**Expected:** Response is `{"status":"ok"}`. Daemon log shows threshold updated to 0.75 (or a reload event logged). No restart.

---

### TC-09: Pipeline not running — control returns error

**Steps:**
1. Start daemon, then immediately kill the pipeline thread (or test via the runtime_commands integration test `daemon_handle_without_runtime_tx_returns_error`)
2. Send any RuntimeCommand via socket

**Expected:** Response is `{"status":"error","message":"pipeline not running"}`. No panic.

---

### TC-10: Automated test suite (regression gate)

**Steps:**
1. `cargo test --test control_protocol`
2. `cargo test --test runtime_commands`

**Expected:** 17/17 and 6/6 pass respectively. Zero failures.
