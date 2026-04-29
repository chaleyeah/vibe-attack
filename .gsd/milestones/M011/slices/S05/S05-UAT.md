# S05: Publish GitHub Release v1.0.0 — UAT

**Milestone:** M011
**Written:** 2026-04-29T11:47:29.181Z

# S05 UAT — Publish GitHub Release v1.0.0

## Preconditions
- `gh` CLI authenticated with `workflow` scope (`gh auth status` confirms)
- `git ls-remote --tags origin v1.0.0` returns a ref (tag is live on origin)
- Workflow run 25088427524 has conclusion = "success"

## Test Cases

### TC-01: Release exists and is non-draft with correct tag
```
gh release view v1.0.0 --json tagName,isDraft --jq '{tag: .tagName, draft: .isDraft}'
```
**Expected:** `{"tag":"v1.0.0","draft":false}`

### TC-02: Release has exactly 5 assets with correct names
```
gh release view v1.0.0 --json assets --jq '[.assets[].name] | sort'
```
**Expected:** `["hd2-v1.0.0.hdpack","vibe-attack-1.0.0-1.x86_64.rpm","vibe-attack-v1.0.0-x86_64.AppImage","vibe-attack-v1.0.0.tar.gz","vibe-attack_1.0.0-1_amd64.deb"]`

### TC-03: All assets are in uploaded state (not pending/errored)
```
gh release view v1.0.0 --json assets --jq '[.assets[] | select(.state != "uploaded")] | length'
```
**Expected:** `0`

### TC-04: PKGBUILD sha256sums are pinned (no SKIP entries)
```
! grep -q "'SKIP'" packaging/PKGBUILD && echo "PASS: no SKIP entries"
```
**Expected:** `PASS: no SKIP entries`

### TC-05: PKGBUILD contains exactly two 64-char hex hashes
```
grep -oE "'[0-9a-f]{64}'" packaging/PKGBUILD | wc -l
```
**Expected:** `2`

### TC-06: PKGBUILD source[0] hash matches the project tarball
```
curl -sL https://github.com/chaleyeah/vibe-attack/archive/v1.0.0.tar.gz | sha256sum
```
**Expected:** First 64 chars = `da0a2427d4812c274ec5fbaf4fa5dd7e13d4fb0030a484f4e06753b8ff6f4c6c`
(Requires authenticated access since repo is private — use `gh` token via `curl -H "Authorization: token $(gh auth token)"` if needed)

### TC-07: PKGBUILD source[1] hash matches sherpa-onnx prebuilt tarball
```
curl -sL https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2 | sha256sum
```
**Expected:** First 64 chars = `1b95e49f889dee65310cab832d6181db619ea3ac77ecd60fe8b301028145781c`

### TC-08: Packaging tests still pass (no PKGBUILD damage)
```
cargo test --test packaging
```
**Expected:** `test result: ok. 15 passed; 0 failed`

### TC-09: v1.0.0 tag is on origin main HEAD
```
git ls-remote --tags origin v1.0.0
```
**Expected:** Non-empty output containing `refs/tags/v1.0.0`

### Edge Cases
- **Re-tag safety**: Confirm `git tag -l v1.0.0` shows the tag locally — force-moving this tag would invalidate the PKGBUILD source[0] sha256. Do not re-tag post-publish.
- **Private repo asset access**: Authenticated download via `gh release download v1.0.0 --pattern "*.AppImage" --dir /tmp/` should succeed; unauthenticated curl to releases/latest/download/ returns 404 by design.
