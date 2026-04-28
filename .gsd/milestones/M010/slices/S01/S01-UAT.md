# S01: AppImage build verification — UAT

**Milestone:** M010
**Written:** 2026-04-28T03:54:48.817Z

# S01 UAT: AppImage Build Verification

## Preconditions
- Working directory: `/home/chadmin/Github/hd-linux-voice`
- `cargo` is available and `target/release/vibe-attack` can be built
- No assumption about linuxdeploy/appimagetool presence

---

## Test 1: verify-appimage.sh exists and is executable

**Steps:**
1. Run `test -f scripts/verify-appimage.sh && echo EXISTS`
2. Run `test -x scripts/verify-appimage.sh && echo EXECUTABLE`

**Expected:** Both commands print their label and exit 0.

---

## Test 2: verify-appimage.sh produces a valid transcript on a host without packaging tools

**Steps:**
1. Run `bash scripts/verify-appimage.sh /tmp/uat-s01-test.md`
2. Run `grep '^STATUS: ' /tmp/uat-s01-test.md`
3. Run `grep '^DISTRO: ' /tmp/uat-s01-test.md`
4. Run `grep '^KERNEL: ' /tmp/uat-s01-test.md`
5. Run `grep '^SIZE_BYTES: ' /tmp/uat-s01-test.md`
6. Run `grep '^SHA256: ' /tmp/uat-s01-test.md`
7. Run `grep '^EXIT_CODE: ' /tmp/uat-s01-test.md`
8. Run `grep '^VERSION_OUTPUT: ' /tmp/uat-s01-test.md`

**Expected:** Script exits 0. All 7 field lines are present. STATUS is one of: `STATUS: ok`, `STATUS: skipped:tools-missing`, or `STATUS: failed:<reason>`.

---

## Test 3: All three distro transcript files exist with required structure

**Steps:**
1. Run: `for d in debian12 fedora39 arch; do test -f docs/distribution-proofs/appimage/$d/transcript.md && grep -q '^STATUS: ' docs/distribution-proofs/appimage/$d/transcript.md && grep -q '^DISTRO: ' docs/distribution-proofs/appimage/$d/transcript.md && echo "$d OK" || echo "$d FAIL"; done`

**Expected:** All three lines print `OK`.

---

## Test 4: Proof README exists and documents the format

**Steps:**
1. Run `test -f docs/distribution-proofs/appimage/README.md && echo EXISTS`
2. Run `grep -q 'STATUS' docs/distribution-proofs/appimage/README.md && echo MENTIONS_STATUS`
3. Run `grep -q 'pending VM run' docs/distribution-proofs/appimage/README.md && echo MENTIONS_PENDING`

**Expected:** All three commands succeed. README documents the transcript format and the pending-VM-run policy.

---

## Test 5: All three integration test suites pass

**Steps:**
1. Run `cargo test --test distribution_proofs --test packaging --test ui_distribution -- --test-threads=1 2>&1 | tee /tmp/uat-s01-cargo.log`
2. Run `grep -E 'test result: ok\.' /tmp/uat-s01-cargo.log | wc -l`

**Expected:** Command exits 0. Three `test result: ok.` lines appear (distribution_proofs: 6, packaging: 5, ui_distribution: 16 = 27 total).

---

## Test 6: build.sh references both ORT libraries (dual-ORT smoke test)

**Steps:**
1. Run `grep -q 'libonnxruntime.so' packaging/appimage/build.sh && echo ORT_OK`
2. Run `grep -q 'libsherpa-onnx-c-api.so' packaging/appimage/build.sh && echo SHERPA_OK`

**Expected:** Both print their label, confirming the dual-ORT bundling intent is wired into build.sh.

---

## Test 7: fedora39 and arch transcripts contain reproduction instructions

**Steps:**
1. Run `grep -q 'alsa-lib-devel' docs/distribution-proofs/appimage/fedora39/transcript.md && echo FEDORA_PKGS`
2. Run `grep -q 'alsa-lib' docs/distribution-proofs/appimage/appimage/arch/transcript.md || grep -q 'alsa-lib' docs/distribution-proofs/appimage/arch/transcript.md && echo ARCH_PKGS`

**Expected:** Both lines confirm distro-specific package lists are embedded in the pending transcripts.

---

## Edge Cases

- **AppImage > 50 MB:** Run `bash scripts/verify-appimage.sh /tmp/edge-test.md` after artificially inflating the AppImage — expect `STATUS: failed:size-exceeded` and `FAILURE_REASON:` field in transcript, script exits non-zero.
- **AppImage missing:** Remove AppImage then run script — expect `STATUS: failed:appimage-not-found`, transcript written, non-zero exit.
- **--version wrong output:** AppImage returning wrong version string — expect `STATUS: failed:version-mismatch` and partial transcript with actual output in VERSION_OUTPUT field.
