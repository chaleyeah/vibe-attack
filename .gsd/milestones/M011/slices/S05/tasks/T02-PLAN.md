---
estimated_steps: 28
estimated_files: 1
skills_used: []
---

# T02: Pin packaging/PKGBUILD sha256sums to real v1.0.0 release hashes

Replace the two `'SKIP'` entries in `packaging/PKGBUILD`'s `sha256sums` array with real sha256 hex digests for source[0] (the project tarball at `https://github.com/chaleyeah/vibe-attack/archive/v1.0.0.tar.gz`) and source[1] (the sherpa-onnx 1.12.39 prebuilt linux-x64 shared-lib tarball at `https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2`).

**Why:** AUR submissions cannot publish with `sha256sums=('SKIP','SKIP')` — `makepkg` requires real digests so downstream users get integrity verification on source fetch. This task closes the AUR-readiness gap left by S04 (which intentionally deferred this until a real tag was live, per MEM093). It is the final piece of M011's distribution-readiness goal.

**Pre-condition:** T01 must have succeeded — the release at `https://github.com/chaleyeah/vibe-attack/archive/v1.0.0.tar.gz` MUST return HTTP 200. Verify with `curl -sI -L https://github.com/chaleyeah/vibe-attack/archive/v1.0.0.tar.gz | grep -E '^HTTP/' | tail -1` — expect 200. If it returns 404, T01 has not actually completed; stop and report.

**Compute hashes:**
1. Project tarball: `curl -sL https://github.com/chaleyeah/vibe-attack/archive/v1.0.0.tar.gz | sha256sum` — capture the 64-char hex prefix. Note: GitHub's auto-generated `archive/v<tag>.tar.gz` is byte-deterministic per (commit-sha, format), so this hash is stable as long as the tag is not force-moved. (We will not force-move it.)
2. sherpa-onnx tarball: `curl -sL https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2 | sha256sum` — capture the 64-char hex prefix. This is a fixed upstream artifact, sha256 is stable.

**Edit PKGBUILD:**
3. Open `packaging/PKGBUILD` lines 21-22:
   ```
   sha256sums=('SKIP'
               'SKIP')
   ```
   Replace with:
   ```
   sha256sums=('<project-hash>'
               '<sherpa-onnx-hash>')
   ```
   Preserve indentation and quoting style — `pacman` parses this as a bash array; quotes must remain single quotes; the second entry's leading whitespace must align under the first's opening quote per PKGBUILD convention. Match the exact column the existing `'SKIP'` entries occupy.
4. Do NOT change any other field in the file — `pkgname`, `pkgver`, `source`, etc. remain untouched.

**Verify:**
5. `grep -A1 '^sha256sums' packaging/PKGBUILD` — the two array entries must each be 64 hex chars (regex `^[0-9a-f]{64}$`), no `SKIP`, no truncation.
6. `cargo test --test packaging` — must continue to report 15 passed (S04 baseline). The packaging tests do not currently assert PKGBUILD sha256 content, so they should remain unaffected; this check guards against accidental damage to other PKGBUILD-related assertions.
7. (Optional sanity, not required for done) — re-run `curl -sL <url> | sha256sum` and compare; the digest must match what's now in PKGBUILD.

**Failure modes to handle:**
- Project archive curl returns 404 → T01 is not complete; stop.
- sherpa-onnx URL returns non-200 (upstream removed the asset) → unlikely; if it happens, report as a real blocker — pinning a fake hash would brick the AUR build.
- A future force-move of the v1.0.0 tag would invalidate the project hash. Document the assumption: tags are immutable post-publish (project convention); we are NOT planning to re-tag.

**Out of scope:** Actually submitting the PKGBUILD to AUR (`mkaurball`, `git push aur`) is operator runbook work documented in `docs/distribution-proofs/aur/README.md` and is M011/S02 (or later) territory, not S05. S05 only pins the hashes so the PKGBUILD is publishable.

## Inputs

- ``packaging/PKGBUILD``
- ``tests/packaging.rs``

## Expected Output

- ``packaging/PKGBUILD` (sha256sums=('<64hex>','<64hex>'); no 'SKIP')`

## Verification

grep -E "^\s*'[0-9a-f]{64}'" packaging/PKGBUILD | wc -l reports 2; ! grep -q "'SKIP'" packaging/PKGBUILD; cargo test --test packaging reports 'test result: ok. 15 passed'.

## Observability Impact

No runtime boundary touched in code — this is a manifest-only edit. Failure visibility relies on `cargo test --test packaging` (S04 baseline) plus a grep assertion on the file itself. No observability surfaces added.
