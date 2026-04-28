---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Add tests/distribution_proofs.rs asserting transcript structure

Add a new integration test file at tests/distribution_proofs.rs that asserts the three per-distro transcript files exist and have the required structural fields (STATUS, DISTRO, KERNEL, SIZE_BYTES, SHA256, VERSION_OUTPUT, EXIT_CODE). The test must NOT assert on STATUS=ok specifically — it must accept STATUS: ok, STATUS: skipped:tools-missing, or STATUS: pending VM run as valid values, since the slice's contract is that the proof harness is in place and structurally complete, not that all three VM runs have been executed by an autonomous agent. Also add a test that asserts scripts/verify-appimage.sh exists, is marked executable, contains the STATUS line emitter, and that build.sh references both libonnxruntime.so and libsherpa-onnx-c-api.so (smoke-test on dual-ORT bundling intent — extends the existing tests/packaging.rs coverage). Add the test file to the existing CI test invocation (it runs automatically with `cargo test --test distribution_proofs`). Run `cargo test --test packaging --test ui_distribution --test distribution_proofs -- --test-threads=1` and confirm all pass. Reason for serial execution: MEM005 / MEM074 documented shared-tmpdir flake under parallel execution.

## Inputs

- ``docs/distribution-proofs/appimage/debian12/transcript.md` — output of T02; asserted by the new test`
- ``docs/distribution-proofs/appimage/fedora39/transcript.md` — output of T02; asserted by the new test`
- ``docs/distribution-proofs/appimage/arch/transcript.md` — output of T02; asserted by the new test`
- ``scripts/verify-appimage.sh` — output of T01; existence + structure asserted by the new test`
- ``tests/packaging.rs` — existing static test file; the new test file mirrors its style`

## Expected Output

- ``tests/distribution_proofs.rs` — new integration test asserting transcript structure and verify-appimage.sh presence; runs in CI alongside existing packaging tests`

## Verification

cargo test --test distribution_proofs --test packaging --test ui_distribution -- --test-threads=1 2>&1 | tee /tmp/s01-tests.log && grep -E 'test result: ok\.' /tmp/s01-tests.log | wc -l | grep -q '^3$'

## Observability Impact

Signals added/changed: cargo test now reports 3+ assertions on distribution proof structure. How a future agent inspects this: `cargo test --test distribution_proofs -- --nocapture` shows exactly which transcript fields are missing. Failure state exposed: a missing transcript or missing field fails CI loudly with the file path and field name in the assertion message.
