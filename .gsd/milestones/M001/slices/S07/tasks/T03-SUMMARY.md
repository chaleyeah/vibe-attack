---
id: T03
parent: S07
milestone: M001
key_files:
  - docs/troubleshooting.md
  - docs/configuration.md
key_decisions:
  - Replaced conflict warnings with positive deployment guidance rather than simply deleting them — users still need to know about .so runtime dependencies
  - ORT_DYLIB_PATH guidance placed in both files for discoverability (troubleshooting for custom installs, configuration for context when enabling wake-word)
duration: 
verification_result: passed
completed_at: 2026-04-25T20:11:23.289Z
blocker_discovered: false
---

# T03: Replaced stale ORT dual-instance conflict warnings with shared .so deployment guidance and ORT_DYLIB_PATH instructions in troubleshooting.md and configuration.md

**Replaced stale ORT dual-instance conflict warnings with shared .so deployment guidance and ORT_DYLIB_PATH instructions in troubleshooting.md and configuration.md**

## What Happened

Both documentation files contained stale warnings telling users to enable only one ONNX Runtime feature at a time to avoid conflicts. Now that T01 switched sherpa-onnx to shared ORT linking, these warnings are misleading and needed to be replaced with accurate deployment guidance.

In `docs/troubleshooting.md` (Models section, lines 89-91), the "Two concurrent ONNX Runtime instances can conflict" paragraph was replaced with a note explaining that `libonnxruntime.so` and `libsherpa-onnx-c-api.so` are placed next to the binary at build time, and providing the `ORT_DYLIB_PATH` environment variable for custom installs. A new "Shared library deployment" paragraph was added at the end of the Build section documenting that `cargo build` copies both `.so` files into `target/` and that they must travel with the binary when packaging or deploying.

In `docs/configuration.md` (wake section, lines 172-173), the blockquote warning about simultaneous STT and wake-word conflicts was replaced with a blockquote note confirming that both features can now run simultaneously via shared library linking, with `ORT_DYLIB_PATH` guidance for non-standard installs.

`tests/documentation.rs` was verified to contain zero assertions referencing the removed conflict text ("only one", "conflict", "bad_alloc"), so no test changes were needed. All four plan verification checks pass via Grep tool confirmation.

## Verification

Verified via Grep tool:
1. `grep -qi 'ensure only one feature' docs/troubleshooting.md` → 0 matches (PASS)
2. `grep -qi 'enable only one at a time' docs/configuration.md` → 0 matches (PASS)
3. `grep -qi 'ORT_DYLIB_PATH' docs/troubleshooting.md` → 2 matches (PASS)
4. `grep -qi 'libonnxruntime' docs/configuration.md` → 2 matches (PASS)
5. Static check of tests/documentation.rs: no assertions reference "only one", "conflict", "bad_alloc", or "ensure only" → 0 matches (PASS)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -qi 'ensure only one feature' docs/troubleshooting.md (count)` | 1 | ✅ pass (0 matches — old conflict text removed) | 15ms |
| 2 | `grep -qi 'enable only one at a time' docs/configuration.md (count)` | 1 | ✅ pass (0 matches — old conflict note removed) | 12ms |
| 3 | `grep -qi 'ORT_DYLIB_PATH' docs/troubleshooting.md (count)` | 0 | ✅ pass (2 matches — guidance present) | 11ms |
| 4 | `grep -qi 'libonnxruntime' docs/configuration.md (count)` | 0 | ✅ pass (2 matches — deployment note present) | 10ms |
| 5 | `grep -qi 'only one|conflict|bad_alloc' tests/documentation.rs (count)` | 1 | ✅ pass (0 matches — no test assertions reference removed text) | 13ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `docs/troubleshooting.md`
- `docs/configuration.md`
