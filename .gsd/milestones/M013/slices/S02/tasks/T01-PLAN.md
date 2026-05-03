---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Add validate-pkgbuild job to ci.yml

Add a new job that bash -n checks PKGBUILD syntax and sources it to verify required fields (pkgname, pkgver, pkgrel, arch, license) and function definitions (build, package) are present

## Inputs

- `.github/workflows/ci.yml`
- `packaging/PKGBUILD`

## Expected Output

- `validate-pkgbuild job present in ci.yml`
- `job runs on ubuntu-22.04 without Arch container`
- `checks pkgname pkgver pkgrel arch license build() package()`

## Verification

ci.yml contains validate-pkgbuild job; bash -n packaging/PKGBUILD exits 0 locally
