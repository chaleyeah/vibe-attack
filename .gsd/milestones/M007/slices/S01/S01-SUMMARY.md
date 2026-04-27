---
id: S01
parent: M007
milestone: M007
provides:
  - ["sha2 absent from Cargo.toml direct dependencies", "DispatcherState visibility narrowed to pub(crate)", "load_profiles reads {name}/pack.yaml subdirectory format matching handle_switch_profile and Pack::load_from_dir", "tests/profile_listing.rs integration test pins load_profiles behavior", "cargo test (40 lib + integration) passes clean", "success-criteria grep: exactly 1 hit (control/mod.rs CancellationToken TODO)"]
requires:
  []
affects:
  []
key_files:
  - ["Cargo.toml", "Cargo.lock", "src/pipeline/dispatcher.rs", "src/ui/config_app.rs", "tests/profile_listing.rs"]
key_decisions:
  - ["sha2 remains in Cargo.lock as transitive dep of zip/pbkdf2 — only the direct-dependency pin was removed, no lock churn beyond that", "DispatcherState struct, flags field, and all three pub methods narrowed to pub(crate) — impl Default requires no annotation", "load_profiles rewritten to filter by is_dir() && path.join('pack.yaml').exists() — existing flat-file tests rewritten rather than left as always-passing dead fixtures", "Integration test uses XDG_CONFIG_HOME env-var override (same pattern as unit tests) to redirect load_profiles at tempdir — no API change needed", "serial_test::serial used in integration test to guard XDG_CONFIG_HOME env-var races", "cargo clippy unavailable locally — RUSTFLAGS=-D warnings cargo check --all-targets used as substitute; CI runs authoritative clippy"]
patterns_established:
  - ["Profile loading convention: {name}/pack.yaml subdirectory format. All three surfaces (load_profiles, handle_switch_profile, Pack::load_from_dir) now agree.", "Integration tests for XDG-dependent functions use XDG_CONFIG_HOME override + serial_test::serial — no API changes needed.", "Full test suite must run with --test-threads=1 locally to avoid test_pack_export_import_with_sounds tmpdir-pollution flake."]
observability_surfaces:
  - ["none"]
drill_down_paths:
  - [".gsd/milestones/M007/slices/S01/tasks/T01-SUMMARY.md", ".gsd/milestones/M007/slices/S01/tasks/T02-SUMMARY.md", ".gsd/milestones/M007/slices/S01/tasks/T03-SUMMARY.md", ".gsd/milestones/M007/slices/S01/tasks/T04-SUMMARY.md", ".gsd/milestones/M007/slices/S01/tasks/T05-SUMMARY.md"]
duration: ""
verification_result: passed
completed_at: 2026-04-27T11:39:26.766Z
blocker_discovered: false
---

# S01: Dead code, dead deps, and the load_profiles bug fix

**Removed sha2 dead dependency, narrowed DispatcherState to pub(crate), fixed load_profiles to use the canonical {name}/pack.yaml subdirectory format, and pinned the fix with a new hermetic integration test — all 40+ tests pass clean.**

## What Happened

S01 addressed four targeted cleanup items: removing the unused sha2 direct dependency, narrowing internal visibility on DispatcherState, closing a latent UI-vs-control-plane inconsistency in profile loading, and adding an integration test to pin the corrected behavior.

**T01 — sha2 removal:** grep confirmed zero uses of sha2 in src/ and tests/. The `sha2 = "0.10"` line was removed from Cargo.toml [dependencies]. sha2 remains in Cargo.lock as a transitive dep of zip (via pbkdf2) — this is expected and requires no action. cargo check and cargo test (40 tests) both passed clean.

**T02 — DispatcherState visibility:** All 5 references to DispatcherState live inside src/pipeline/dispatcher.rs with none in tests/ or other modules. The struct, its `flags` field, and the three pub methods (new, get, set) were narrowed to pub(crate). The impl Default block requires no visibility annotation. cargo check passed in 0.36s; full suite passed (the pre-existing test_pack_export_import_with_sounds parallel-ordering flake is unrelated and passes in isolation).

**T03 — load_profiles fix:** The existing implementation scanned for flat *.yaml files in the profiles directory and returned their file stems. This was inconsistent with Pack::load_from_dir and handle_switch_profile, which both expect {name}/pack.yaml subdirectory format. The new filter_map predicate checks `entry.file_type().ok()?.is_dir() && path.join("pack.yaml").exists()` and returns the directory name. The two tests that validated old flat-file behavior were rewritten to use subdirectory fixtures. All 4 config_app tests pass.

**T04 — Integration test:** tests/profile_listing.rs creates a tempdir with three fixture entries: (a) good_profile/pack.yaml (valid), (b) flat_profile.yaml (flat file at root), (c) no_pack/ (dir without pack.yaml). It sets XDG_CONFIG_HOME to redirect load_profiles() at the tempdir, calls vibe_attack::ui::config_app::load_profiles(), and asserts the result is exactly ["good_profile"]. serial_test::serial guards against XDG_CONFIG_HOME env-var races. Test passed 1/1 on first run.

**T05 — Full verification:** cargo test -- --test-threads=1 → 40 passed, 0 failed. cargo test --features gui -- --test-threads=1 → 43 passed, 0 failed (3 extra GUI-gated tests active). RUSTFLAGS="-D warnings" cargo check --all-targets → 0 warnings (both default and gui feature gates). Success-criteria grep returned exactly 1 hit: src/control/mod.rs:129 — the documented, justified CancellationToken TODO that is the only acceptable remaining hit per the slice contract.

After S01, all three profile-touching surfaces (load_profiles, handle_switch_profile, Pack::load_from_dir) agree on {name}/pack.yaml subdirectory format. The user-facing inconsistency where the UI listed profiles the switch handler could never load is closed.

## Verification

cargo test -- --test-threads=1 → exit 0, 40 passed, 0 failed; cargo test --features gui -- --test-threads=1 → exit 0, 43 passed, 0 failed; RUSTFLAGS="-D warnings" cargo check --all-targets → exit 0, 0 warnings; RUSTFLAGS="-D warnings" cargo check --all-targets --features gui → exit 0, 0 warnings; grep -rn 'hd.linux.voice|hd_linux_voice|hd2_linux|TODO|FIXME|HACK|dead_code|allow(unused' src/ → exactly 1 hit: src/control/mod.rs:129 (documented CancellationToken TODO, the only acceptable remaining hit per slice contract). cargo clippy not available locally — CI runs the authoritative clippy check via rustup-provisioned toolchain.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

["cargo clippy not available on this system — RUSTFLAGS=-D warnings cargo check --all-targets used as substitute for both default and gui feature sets. CI runs the authoritative clippy check via rustup-provisioned toolchain."]

## Known Limitations

["test_pack_export_import_with_sounds is a pre-existing parallel-ordering flake (tmpdir pollution); passes cleanly with --test-threads=1 and in isolation. Not introduced by S01."]

## Follow-ups

None.

## Files Created/Modified

None.
