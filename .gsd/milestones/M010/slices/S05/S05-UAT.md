# S05: README install section rewrite — UAT

**Milestone:** M010
**Written:** 2026-04-28T11:17:45.721Z

# S05: README install section rewrite — UAT

**Milestone:** M010
**Written:** 2026-04-28

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: This slice changes exactly one file (README.md). Correctness is fully verifiable via grep assertions (required content present) and cargo regression tests (no code broken). No runtime behavior changed.

## Preconditions

- `README.md` has been updated by T01
- The repository is at the state after T01 completion
- `cargo` is available and the project builds

## Smoke Test

```bash
grep -q '^### AppImage' README.md && grep -q '^### AUR' README.md && grep -q '^### First-Run Wizard' README.md && grep -q '^### Build from Source' README.md && echo "SMOKE PASS"
```
Expected: `SMOKE PASS`

## Test Cases

### 1. AppImage subsection is present and correct

```bash
grep -q '^### AppImage' README.md && echo OK
grep -q 'libfuse2' README.md && echo OK
grep -q 'releases' README.md && echo OK
```
1. Run the three greps above.
2. **Expected:** All three print `OK` (exit 0). The AppImage subsection exists, calls out libfuse2 (not libfuse3), and links to the Releases page.

### 2. AUR subsection is present and correct

```bash
grep -q '^### AUR' README.md && echo OK
grep -q 'paru -S vibe-attack\|yay -S vibe-attack' README.md && echo OK
grep -q 'onnxruntime' README.md && echo OK
```
1. Run the three greps above.
2. **Expected:** All three print `OK`. AUR section exists, shows both paru and yay commands, notes onnxruntime runtime dep.

### 3. First-Run Wizard subsection is present and complete

```bash
grep -q '^### First-Run Wizard' README.md && echo OK
grep -q -- '--skip-wizard' README.md && echo OK
grep -q 'docs/uinput-setup.md' README.md && echo OK
```
1. Run the three greps.
2. **Expected:** All three print `OK`. Wizard section exists, documents --skip-wizard flag, and links to docs/uinput-setup.md (not inlined).

### 4. Build from Source subsection preserved and extended

```bash
grep -q '^### Build from Source' README.md && echo OK
grep -q 'alsa-lib-devel' README.md && echo OK
```
1. Run both greps.
2. **Expected:** Both print `OK`. Build-from-source subsection exists and contains Fedora-specific deps (alsa-lib-devel).

### 5. Regression guard — all packaging and distribution tests still pass

```bash
cargo test --test ui_distribution --test packaging --test distribution_proofs --test wizard_proofs -- --test-threads=1
```
1. Run the cargo test command above.
2. **Expected:** `test result: ok. 41 passed; 0 failed`. No test regressions introduced by the README change.

### 6. Section ordering — AppImage before AUR before Wizard before Build from Source

```bash
awk '/^### AppImage/{a=NR} /^### AUR/{b=NR} /^### First-Run Wizard/{c=NR} /^### Build from Source/{d=NR} END{if(a<b && b<c && c<d) print "ORDER OK"; else print "ORDER FAIL"}' README.md
```
1. Run the awk command above.
2. **Expected:** `ORDER OK`. The four subsections appear in the correct sequence.

## Edge Cases

### --skip-wizard flag documented

```bash
grep -q -- '--skip-wizard' README.md && echo OK
```
1. Run the grep.
2. **Expected:** `OK`. Users who pre-populate config.yaml know they can bypass the wizard.

### libfuse2 vs libfuse3 — correct package named

```bash
grep 'libfuse' README.md
```
1. Run the grep and inspect output.
2. **Expected:** Output contains `libfuse2`; `libfuse3` should NOT appear. Installing libfuse3 instead of libfuse2 is a known breakage mode on Debian/Ubuntu.

## Failure Signals

- Any grep assertion returning exit 1 — required content is missing
- `ORDER FAIL` from the section-ordering check — user would read wrong sequence
- Any cargo test failure — README change inadvertently broke a packaging or distribution proof

## Not Proven By This UAT

- That a real human stranger can actually follow the README without confusion (human experience UAT would require a live tester unfamiliar with the project)
- That the Releases page URL resolves to a downloadable AppImage (no AppImage release tag may exist yet — S06 covers live download verification)
- Runtime wizard behavior (covered by S02 UAT)
- AUR install on a live Arch system (covered by S04 UAT)

## Notes for Tester

The libfuse2 check is the highest-stakes assertion: getting libfuse3 documented instead of libfuse2 would silently break AppImage launches on Debian/Ubuntu. Confirm the grep output explicitly shows `libfuse2`.

The Releases page may show no assets yet if S03 CI has not run a tag push — that is expected at this stage. The link itself is correct and will become live once the first release tag is pushed.
