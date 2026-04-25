# T06: 02-pipeline-core 06

**Slice:** S02 — **Milestone:** M001

## Description

Close the “target hardware proof artifacts” gap by adding an in-repo place to store evidence and a template that captures the required outputs (stdout JSONL, stderr timing logs, and computed p95) so Phase 2 can be marked complete after a single human run.

Purpose: Verification requires committed evidence, not just a runbook.
Output: Proof artifact templates + doc wiring to the baseline procedure.

## Must-Haves

- [ ] "The repo contains an in-repo template + instructions for archiving target hardware proof artifacts (transcript.jsonl, timing.log, computed p95)."
- [ ] "A human can run the procedure once on target hardware and commit the evidence in a consistent location without inventing new structure."

## Files

- `docs/latency-baseline.md`
- `docs/latency-proofs/phase-02-target-hardware/README.md`
- `docs/latency-proofs/phase-02-target-hardware/RESULTS.template.md`
