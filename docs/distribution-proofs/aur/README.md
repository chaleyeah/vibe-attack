STATUS: pending submission
MAINTAINER: chaleyeah
AUR_PACKAGE: vibe-attack
AUR_URL: https://aur.archlinux.org/packages/vibe-attack

---

# AUR Submission Workflow — vibe-attack

This document records the release-time steps to pin `pkgver`/`sha256sums`, verify the
package builds cleanly, and push to the AUR as maintainer `chaleyeah`.

---

## 1. Pre-Submission Checklist

### 1a. Pin `pkgver` in `packaging/PKGBUILD`

Set `pkgver` to the release tag **without** the `v` prefix:

```
pkgver=0.1.0   # matches git tag v0.1.0
pkgrel=1       # reset to 1 for a new upstream release; increment for packaging-only fixes
```

### 1b. Compute and pin `sha256sums`

Two source entries must have real hashes before submission. The `'SKIP'` placeholders
used during development must be replaced.

**Source 0 — project source tarball:**

```bash
sha256sum <(curl -fsSL "https://github.com/chaleyeah/vibe-attack/archive/v0.1.0.tar.gz") \
  | awk '{print $1}'
```

Or fetch first, then hash:

```bash
curl -fsSL -o /tmp/vibe-attack-0.1.0.tar.gz \
  "https://github.com/chaleyeah/vibe-attack/archive/v0.1.0.tar.gz"
sha256sum /tmp/vibe-attack-0.1.0.tar.gz
```

**Source 1 — sherpa-onnx 1.12.39 prebuilt shared-lib archive:**

```bash
curl -fsSL -o /tmp/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2 \
  "https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2"
sha256sum /tmp/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2
```

**Apply both hashes to `sha256sums` in `packaging/PKGBUILD`:**

```bash
# Either edit manually, or use updpkgsums (from pacman package-query):
cd packaging
updpkgsums   # fetches sources, recomputes, rewrites sha256sums in-place
```

After running `updpkgsums`, verify the result looks like:

```
sha256sums=('aabbcc...  (64 hex chars)'
            'ddeeff...  (64 hex chars)')
```

---

## 2. Verification Checklist

Run all steps before pushing to AUR.

### 2a. namcap lint

```bash
cd packaging
namcap PKGBUILD
```

Expected: no errors or warnings. Common issues to watch for:
- Missing/unnecessary `depends` entries
- Non-standard install paths

### 2b. makepkg in a clean Arch chroot

Use `extra-x86_64-build` (from `devtools` package) for a pristine environment:

```bash
cd packaging
extra-x86_64-build
```

Or manually with `mkarchroot`:

```bash
mkarchroot /tmp/cleanroot base-devel
makechrootpkg -c -r /tmp/cleanroot
```

The build must succeed with exit code 0 and produce a `.pkg.tar.zst` file.

### 2c. Offline build verification

After sources are fetched once, confirm makepkg succeeds without network:

```bash
cd packaging
makepkg --offline --noextract   # reuses already-fetched sources
# OR clean and re-fetch, then block network:
makepkg --nobuild               # just downloads
unshare -n makepkg --noextract  # runs build with no network namespace
```

### 2d. Runtime smoke test

After `makepkg -si` (or `makepkg` + `sudo pacman -U *.pkg.tar.zst`):

```bash
vibe-attack --help
vibe-attack-config --help
```

Both commands must print help text and exit 0.

---

## 3. AUR Submission Steps

### 3a. First-time setup (one-time)

Ensure your AUR SSH key is configured:

```bash
# ~/.ssh/config entry:
Host aur.archlinux.org
    IdentityFile ~/.ssh/aur
    User aur
```

Generate and upload the key at https://aur.archlinux.org/account/chaleyeah if not done.

### 3b. Clone or init the AUR repo

**First submission (package doesn't exist yet):**

```bash
git clone ssh://aur@aur.archlinux.org/vibe-attack.git aur-vibe-attack
# Will be empty — that is expected for a new package
```

**Subsequent updates:**

```bash
cd aur-vibe-attack
git pull
```

### 3c. Copy PKGBUILD and generate .SRCINFO

```bash
cp packaging/PKGBUILD aur-vibe-attack/PKGBUILD
cd aur-vibe-attack
makepkg --printsrcinfo > .SRCINFO
```

Inspect `.SRCINFO` — it must list both source entries with real sha256sums (not `SKIP`).

### 3d. Commit and push

```bash
cd aur-vibe-attack
git add PKGBUILD .SRCINFO
git commit -m "vibe-attack 0.1.0-1"
git push origin master
```

The package will be visible at https://aur.archlinux.org/packages/vibe-attack within
a few minutes.

### 3e. Post-submission verification

```bash
# Confirm the web page is live:
curl -sI https://aur.archlinux.org/packages/vibe-attack | head -1
# Expected: HTTP/2 200

# Test install via an AUR helper:
yay -S vibe-attack
```

---

## Notes

- `onnxruntime` is a genuine runtime dependency. `libsherpa-onnx-c-api.so` links
  `libonnxruntime.so` with `RPATH=$ORIGIN`, which only resolves when both `.so` files
  are co-located (AppImage). In the native Arch package only binaries land in `/usr/bin/`,
  so the system `onnxruntime` package must provide `/usr/lib/libonnxruntime.so`.
- The `SHERPA_ONNX_ARCHIVE_DIR="$srcdir"` export in `build()` tells `sherpa-onnx-sys`
  to use the pre-fetched source[1] archive instead of downloading during `cargo build`.
- When upgrading the sherpa-onnx version, update both `source[1]` URL and the
  `sha256sums[1]` entry, and re-export `SHERPA_ONNX_ARCHIVE_DIR` with the new path.
