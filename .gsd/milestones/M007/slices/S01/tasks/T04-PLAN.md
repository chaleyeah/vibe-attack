---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T04: Add integration test for load_profiles canonical format

Add a new integration test (tests/profile_listing.rs or extend an existing test file in tests/) that creates a tempdir profiles directory containing: (a) one subdirectory named 'good_profile' containing a valid pack.yaml, (b) one flat file named 'flat_profile.yaml', (c) one empty subdirectory named 'no_pack' with no pack.yaml. Call load_profiles (or extract a testable helper if needed) and assert the returned list contains exactly ['good_profile'] and excludes both 'flat_profile' and 'no_pack'. Document the test rationale in a comment referencing the M007/S01 fix.

## Inputs

- `The fixed load_profiles from T03; tempdir or tempfile dependency already present in [dev-dependencies]`

## Expected Output

- `tests/profile_listing.rs with one passing test asserting subdirectory-only behavior`

## Verification

cargo test --test profile_listing passes; the test creates the three fixture entries, calls the loader, and asserts only 'good_profile' is returned
