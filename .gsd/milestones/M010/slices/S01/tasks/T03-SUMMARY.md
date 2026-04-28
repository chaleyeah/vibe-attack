---
id: T03
parent: S01
milestone: M010
key_files:
  - tests/distribution_proofs.rs
key_decisions:
  - Tests accept STATUS: ok / skipped:tools-missing / pending VM run as valid — structural completeness is enforced, not execution completeness, matching the slice contract that proof harness is in place before VM runs
  - Executable bit check uses std::os::unix::fs::PermissionsExt mode & 0o111 (any execute bit) rather than exact mode match, to tolerate different umask settings across distros
duration: 
verification_result: passed
completed_at: 2026-04-28T03:52:56.884Z
blocker_discovered: false
---

# T03: Add tests/distribution_proofs.rs: 6 integration tests asserting per-distro transcript structure and verify-appimage.sh integrity; all 3 test suites pass (27 tests total)

**Add tests/distribution_proofs.rs: 6 integration tests asserting per-distro transcript structure and verify-appimage.sh integrity; all 3 test suites pass (27 tests total)**

## What Happened

Created `tests/distribution_proofs.rs` with 6 tests covering the two verification surfaces from the slice plan:

**Transcript structure tests (3):** One test per distro (debian12, fedora39, arch) — each reads the corresponding `docs/distribution-proofs/appimage/<distro>/transcript.md` and asserts all 7 required fields are present (STATUS, DISTRO, KERNEL, SIZE_BYTES, SHA256, EXIT_CODE, VERSION_OUTPUT) and that STATUS carries one of the three valid values: `ok`, `skipped:tools-missing`, or `pending VM run`. This deliberately avoids asserting `STATUS: ok` so the test passes on the current build host (debian12=skipped:tools-missing, fedora39/arch=pending) while still enforcing structural completeness.

**verify-appimage.sh integrity tests (2):** Asserts the script exists, is marked executable (mode & 0o111 != 0), and contains both a `STATUS:` field reference and an `echo "STATUS:` emitter — confirming the transcript format is hardwired into the script, not ad-hoc.

**build.sh dual-ORT smoke test (1):** Asserts `build.sh` references both `libonnxruntime.so` and `libsherpa-onnx-c-api.so`, extending the T01 packaging.rs coverage to cover the sherpa-onnx bundling path.

No Cargo.toml changes were needed — integration tests in `tests/` are auto-discovered by cargo. The test file mirrors the style of `tests/packaging.rs` (read_file helper via CARGO_MANIFEST_DIR, direct assert! with descriptive messages).

## Verification

Ran `cargo test --test distribution_proofs --test packaging --test ui_distribution -- --test-threads=1` (serial per MEM005/MEM074 shared-tmpdir flake). All 3 test suites passed: distribution_proofs (6/6), packaging (5/5), ui_distribution (16/16). The count gate `grep -E 'test result: ok\.' | wc -l | grep -q '^3$'` also passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test distribution_proofs --test packaging --test ui_distribution -- --test-threads=1 2>&1 | tee /tmp/s01-tests.log` | 0 | ✅ pass — 27 tests across 3 suites, 0 failed | 3200ms |
| 2 | `grep -E 'test result: ok\.' /tmp/s01-tests.log | wc -l | grep -q '^3$'` | 0 | ✅ pass — exactly 3 'test result: ok.' lines | 10ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/distribution_proofs.rs`
