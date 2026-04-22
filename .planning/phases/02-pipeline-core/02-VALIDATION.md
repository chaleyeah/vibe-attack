---
phase: 2
slug: pipeline-core
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-22
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`cargo test`) |
| **Config file** | none |
| **Quick run command** | `cargo test -q` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds (without model-gated tests) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -q`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-00-01 | 00 | 0 | STT-01 | — | No network access; model path validated at startup | integration (gated) | `RUN_STT_TESTS=1 cargo test --test stt_smoke -- --include-ignored` | ❌ W0 | ⬜ pending |
| 02-00-02 | 00 | 0 | ACT-02 | — | Wake word triggers LISTENING and emits trigger event | integration (gated) | `RUN_KWS_TESTS=1 cargo test --test wake_word -- --include-ignored` | ❌ W0 | ⬜ pending |
| 02-00-03 | 00 | 0 | STT-04 | — | Stage timing fields present and monotonic durations non-negative | unit/integration | `cargo test -q` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/stt_smoke.rs` — gated STT test harness (`RUN_STT_TESTS=1`)
- [ ] `tests/wake_word.rs` — gated wake word harness (`RUN_KWS_TESTS=1`)
- [ ] `tests/jsonl_schema.rs` — JSONL schema + stage timing invariants

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| End-to-end latency < 500ms on target hardware | STT-04 | Hardware-dependent perf measurement | Run the daemon on target hardware, speak short phrases, archive stderr stage timings and stdout JSONL transcript for review |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
