# S04: Version bump + release CI (.deb / .rpm jobs) — UAT

**Milestone:** M011
**Written:** 2026-04-29T01:58:53.500Z

# S04 UAT: Version bump + release CI (.deb / .rpm jobs)

## Preconditions
- Repository is at `/home/chadmin/Github/hd-linux-voice`
- Rust toolchain is available (`cargo`)
- Python3 is available with the `yaml` module

---

## Test Group 1: Version 1.0.0 across all manifests

**TC-S04-01: Cargo.toml version**
1. Run: `grep '^version' Cargo.toml`
2. Expected: `version = "1.0.0"` (line 3)
3. Pass: line reads exactly `version = "1.0.0"`; no `0.1.0` in the active version field

**TC-S04-02: RPM spec version**
1. Run: `grep '^Version:' packaging/vibe-attack.spec`
2. Expected: `Version:        1.0.0`
3. Run: `grep '^%changelog' packaging/vibe-attack.spec -A 3`
4. Expected: first changelog entry contains `1.0.0-1` and date `Tue Apr 28 2026`
5. Pass: active Version field is 1.0.0; historical 0.1.0 entry remains below it

**TC-S04-03: PKGBUILD version and sha256sums**
1. Run: `grep '^pkgver' packaging/PKGBUILD`
2. Expected: `pkgver=1.0.0`
3. Run: `grep -A1 'sha256sums' packaging/PKGBUILD`
4. Expected: `sha256sums=('SKIP', 'SKIP')` — unchanged
5. Pass: version bumped; sums untouched per MEM093

**TC-S04-04: Debian changelog**
1. Run: `head -1 packaging/debian/changelog`
2. Expected: `vibe-attack (1.0.0-1) unstable; urgency=medium`
3. Run: `head -5 packaging/debian/changelog`
4. Expected: timestamp line contains `Tue, 28 Apr 2026`
5. Pass: new stanza prepended; old 0.1.0 stanza preserved below

**TC-S04-05: CHANGELOG.md structure**
1. Run: `grep -n '^\#\# ' CHANGELOG.md | head -5`
2. Expected: first heading is `## [Unreleased]` (empty), second is `## [1.0.0] - 2026-04-28`
3. Run: `grep 'Notes on versioning' CHANGELOG.md`
4. Expected: no output (paragraph removed)
5. Pass: 1.0.0 block dated 2026-04-28 present; Unreleased empty; notes paragraph gone

---

## Test Group 2: release.yml 4-job architecture

**TC-S04-06: Job declarations**
1. Run: `grep '^  build-' .github/workflows/release.yml`
2. Expected: lines `  build-appimage:`, `  build-deb:`, `  build-rpm:` present
3. Run: `grep '^  release:' .github/workflows/release.yml`
4. Expected: `  release:` present
5. Pass: all four jobs declared at column 2

**TC-S04-07: Release job dependencies**
1. Run: `grep -A3 '^  release:' .github/workflows/release.yml | grep 'needs'`
2. Expected: `needs: [build-appimage, build-deb, build-rpm]` (or equivalent YAML list)
3. Pass: release job depends on all three build jobs

**TC-S04-08: Artifact upload globs**
1. Run: `grep 'vibe-attack_\*\.deb\|vibe-attack-\*\.x86_64\.rpm\|vibe-attack-\*-x86_64\.AppImage\|vibe-attack-\*\.tar\.gz\|hd2-\*\.hdpack' .github/workflows/release.yml`
2. Expected: all five globs present
3. Run: `grep 'fail_on_unmatched_files' .github/workflows/release.yml`
4. Expected: `fail_on_unmatched_files: true`
5. Pass: all five artifact globs and fail-on-unmatched guard present

**TC-S04-09: Sherpa-onnx cache parity (MEM089)**
1. Run: `grep -c 'sherpa-onnx-1.12.39-linux-x64' .github/workflows/release.yml`
2. Expected: `3` (one per build job: appimage, deb, rpm)
3. Pass: cache key appears exactly 3 times, ensuring no build job cold-downloads sherpa

**TC-S04-10: YAML validity**
1. Run: `python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/release.yml")); print("valid")'`
2. Expected: prints `valid`, exit 0
3. Pass: release.yml is syntactically valid YAML

---

## Test Group 3: Packaging test suite (automated stopping condition)

**TC-S04-11: All packaging tests pass**
1. Run: `cargo test --test packaging 2>&1`
2. Expected: `test result: ok. 15 passed; 0 failed; 0 ignored`
3. Spot-check new tests present in output:
   - `release_yml_has_build_deb_job ... ok`
   - `release_yml_has_build_rpm_job ... ok`
   - `release_yml_uploads_deb_artifact ... ok`
   - `release_yml_uploads_rpm_artifact ... ok`
   - `release_yml_caches_sherpa_onnx_in_all_release_jobs ... ok`
4. Pass: all 15 tests pass including 5 new S04 assertions and all 10 pre-existing tests

---

## Edge Cases

**EC-S04-01: No active 0.1.0 version strings**
- Run: `grep -rn "0\.1\.0" Cargo.toml packaging/ CHANGELOG.md | grep -v 'sha256sums' | grep -v '%changelog' | grep -v 'vibe-attack (0\.1\.0'`
- Expected: zero hits (all active version fields read 1.0.0; only historical changelog entries contain 0.1.0)

**EC-S04-02: debian/control not modified**
- Run: `grep -i 'version' packaging/debian/control`
- Expected: no version number present (control does not embed a version per packaging convention)

**EC-S04-03: PKGBUILD sha256sums integrity**
- Run: `grep 'sha256sums' packaging/PKGBUILD`
- Expected: `sha256sums=('SKIP', 'SKIP')` — not `('SKIP', 'SKIP')` with real hashes; AUR pinning is S05's job
