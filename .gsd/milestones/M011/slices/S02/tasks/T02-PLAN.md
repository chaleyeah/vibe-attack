---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T02: Run wizard UAT scenarios A–D on all four distros and record SCENARIO_* outcomes

Drive the human operator through wizard UAT on each distro: build `vibe-attack-config` from source (`cargo build --release --bin vibe-attack-config`), then exercise the four documented scenarios from `docs/distribution-proofs/wizard/README.md` and the per-distro Reproduction Notes already inlined in each `wizard/<distro>/transcript.md`: A=fresh install with full wizard + voice firing, B=model-pre-placed (skip InstallModel), C=relaunch shows main config screen, D=`--skip-wizard` flag short-circuits. Each scenario's outcome (`ok` or `failed:<scenario-letter>`) is filled into the corresponding SCENARIO_A/B/C/D field; STRATAGEM_FIRED records voice-firing success from Scenario A. The agent's job is (a) before runs, confirm polkit agent presence is documented in the operator brief; (b) after runs, verify all four wizard transcripts carry `STATUS: ok` with all 10 fields populated, and that `cargo test --test wizard_proofs -- --test-threads=1` reports 5/5 passing. Pitfalls to surface to the operator: polkit dialog stalls on headless/Wayland VMs without an agent; `usermod -aG input` requires logout-relogin (or `newgrp input`) before Scenario A's mic test; HuggingFace 302 redirect on restricted networks — pre-place model and run Scenario B first to validate wizard logic independently of network. Real UI bugs surfaced during runs are NOT fixed in this task — file them as candidates for S03 by appending a `## Findings` block under Reproduction Notes if needed.

## Inputs

- `docs/distribution-proofs/wizard/debian13/transcript.md`
- `docs/distribution-proofs/wizard/ubuntu2604/transcript.md`
- `docs/distribution-proofs/wizard/fedora44/transcript.md`
- `docs/distribution-proofs/wizard/cachyos/transcript.md`
- `docs/distribution-proofs/wizard/README.md`
- `tests/wizard_proofs.rs`

## Expected Output

- `docs/distribution-proofs/wizard/debian13/transcript.md`
- `docs/distribution-proofs/wizard/ubuntu2604/transcript.md`
- `docs/distribution-proofs/wizard/fedora44/transcript.md`
- `docs/distribution-proofs/wizard/cachyos/transcript.md`

## Verification

All four wizard/<distro>/transcript.md files carry STATUS: ok with SCENARIO_A/B/C/D = ok and STRATAGEM_FIRED populated; `cargo test --test wizard_proofs -- --test-threads=1` passes 5/5. Verify with: `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/wizard/$d/transcript.md; done | grep -c '^STATUS: ok$'` returns 4, AND `cargo test --test wizard_proofs -- --test-threads=1 2>&1 | grep -E 'test result: ok\. 5 passed'` matches.

## Observability Impact

Signals added/changed: SCENARIO_A/B/C/D become observable per-scenario verdicts on disk. How a future agent inspects this: read the four wizard transcripts; per-scenario failures appear as `failed:<letter>` not as a single opaque STATUS. Failure state exposed: wizard pitfalls (polkit, input-group, network, evdev) become reproducible from the in-file Reproduction Notes.
