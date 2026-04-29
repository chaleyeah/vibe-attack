---
id: T02
parent: S02
milestone: M011
key_files:
  - docs/distribution-proofs/wizard/debian13/transcript.md
  - docs/distribution-proofs/wizard/ubuntu2604/transcript.md
  - docs/distribution-proofs/wizard/fedora44/transcript.md
  - docs/distribution-proofs/wizard/cachyos/transcript.md
  - tests/wizard_proofs.rs
key_decisions:
  - Wizard UAT scenarios A–D cannot be executed in auto-mode — all require full desktop GUI session, polkit visibility, and human observation of UI state transitions
  - Scenario D (--skip-wizard) source logic verified by code inspection (FirstRunState::from_checks(true,true,true,true)) rather than live UI run
  - Structural tests (5/5) pass throughout; transcripts remain STATUS: pending VM run pending operator runs on all four distros
duration: 
verification_result: mixed
completed_at: 2026-04-29T01:28:21.100Z
blocker_discovered: false
---

# T02: Wizard UAT transcripts structurally verified (5/5 tests pass); all four distros remain STATUS: pending VM run — real GUI + human runs required for Scenarios A–D

**Wizard UAT transcripts structurally verified (5/5 tests pass); all four distros remain STATUS: pending VM run — real GUI + human runs required for Scenarios A–D**

## What Happened

This task requires driving a human operator through four wizard UAT scenarios (A=fresh install + voice firing, B=model pre-placed, C=relaunch skips wizard, D=--skip-wizard flag) on each of the four target distros (Debian 13, Ubuntu 26.04, Fedora 44, CachyOS). All four scenarios require a full desktop GUI session, polkit agent visibility, real audio hardware for Scenario A's mic test, and human observation of UI state transitions. Auto-mode cannot execute these scenarios.

**Why all scenarios are blocked in auto-mode:**

- **Scenario A** requires: full desktop session, polkit dialog visible for uinput setup, real microphone for voice firing, human observation of wizard step progression. The `vibe-attack-config` binary opens a Wayland window and hangs waiting for user interaction — there is no CLI-observable outcome for wizard step results.
- **Scenario B** requires the same GUI session with a pre-placed model file — same blocking constraints.
- **Scenario C** requires launching twice and observing that the wizard does NOT appear on second launch — again requires human GUI observation.
- **Scenario D (`--skip-wizard`)** substitutes `FirstRunState::from_checks(true,true,true,true)` in code, bypassing the wizard and showing the main config screen directly. The flag is parsed from args before GUI init, so `--help` works headlessly. But verifying "main config screen appears" requires visual confirmation of a Wayland window, which auto-mode cannot assert.

**What auto-mode verified:**

1. `--help` flag: exits 0 with correct usage text (no window opened, non-GUI path confirmed).
2. `--skip-wizard` is accepted by the argument parser without error (confirmed via `--help` output listing the flag).
3. The wizard source code logic: `--skip-wizard` calls `FirstRunState::from_checks(true,true,true,true)`, setting all probe flags to true, which causes `is_setup_complete()` to return true immediately — no wizard is shown. This is correct by inspection of `src/bin/vibe-attack-config.rs:184–188`.
4. Structural tests: `cargo test --test wizard_proofs -- --test-threads=1` reports 5/5 passing — all four distro transcripts contain the required 10 fields and a recognized STATUS value.

**Operator runs needed:**

All four distros require a human operator with a full desktop VM:

```
# On each distro VM:
cargo build --release --bin vibe-attack-config
export PATH="$PWD/target/release:$PATH"
LD_LIBRARY_PATH=$PWD/target/release vibe-attack-config  # Scenario A/B/C
LD_LIBRARY_PATH=$PWD/target/release vibe-attack-config --skip-wizard  # Scenario D
```

Key pitfalls documented in `docs/distribution-proofs/wizard/README.md`:
- polkit agent must be running (check: `ps aux | grep polkit`) for Scenario A's uinput step
- `usermod -aG input $USER` requires logout-relogin (or `newgrp input`) before mic test
- HuggingFace CDN 302 redirect — pre-place model for Scenario B if network is restricted
- evdev device selection — use manual entry fallback if PTT capture hangs on multi-keyboard machines

After each run, fill in the 10 fields (STATUS, DISTRO, KERNEL, BINARY, BINARY_VERSION, SCENARIO_A, SCENARIO_B, SCENARIO_C, SCENARIO_D, STRATAGEM_FIRED) from observed results. Set STATUS to `ok` when all four scenarios pass.

## Verification

1. `cargo test --test wizard_proofs -- --test-threads=1` → 5/5 passed ✅ (structural field presence for all four distro transcripts)
2. `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/wizard/$d/transcript.md; done | grep -c '^STATUS: ok$'` → 0 (no real VM runs completed) ❌
3. `LD_LIBRARY_PATH=target/release target/release/vibe-attack-config --help` → exit 0, correct usage text ✅
4. Wizard source code inspection confirms `--skip-wizard` flag wiring is correct (Scenario D logic verified) ✅

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test wizard_proofs -- --test-threads=1 2>&1 | grep 'test result'` | 0 | ✅ pass — test result: ok. 5 passed; 0 failed | 90ms |
| 2 | `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/wizard/$d/transcript.md; done | grep -c '^STATUS: ok$'` | 1 | ❌ partial — 0/4 STATUS: ok (all pending operator VM runs) | 10ms |
| 3 | `LD_LIBRARY_PATH=target/release target/release/vibe-attack-config --help; echo exit:$?` | 0 | ✅ pass — --help exits 0, --skip-wizard flag listed in usage output | 50ms |
| 4 | `grep -n 'skip_wizard\|from_checks' src/bin/vibe-attack-config.rs | head -10` | 0 | ✅ pass — Scenario D logic verified: --skip-wizard calls FirstRunState::from_checks(true,true,true,true) | 10ms |

## Deviations

Auto-mode cannot execute wizard UAT scenarios on any distro. Unlike T01 where the AppImage build + verify-appimage.sh could be run headlessly on the Ubuntu 26.04 host, wizard UAT requires an interactive GUI session with human observation. There is no headless equivalent for scenarios A–D. This is the same constraint that affected T01's non-ubuntu2604 distros — the task is human-bound by design. Transcripts remain at STATUS: pending VM run across all four distros.

## Known Issues

All four distro wizard transcripts (debian13, ubuntu2604, fedora44, cachyos) remain at STATUS: pending VM run. The human operator must boot VMs and run wizard UAT scenarios A–D on each distro before the slice verification gate (4/4 STATUS: ok) can pass. The operator brief and per-distro Reproduction Notes are fully documented in docs/distribution-proofs/wizard/README.md and each transcript's Reproduction Notes section.

## Files Created/Modified

- `docs/distribution-proofs/wizard/debian13/transcript.md`
- `docs/distribution-proofs/wizard/ubuntu2604/transcript.md`
- `docs/distribution-proofs/wizard/fedora44/transcript.md`
- `docs/distribution-proofs/wizard/cachyos/transcript.md`
- `tests/wizard_proofs.rs`
