---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T03: Run final end-to-end UAT on all four distros against the published AppImage and close the proof gate

PRECONDITION: this task is BLOCKED until S04 publishes a real AppImage to the GitHub Releases URL referenced by `docs/distribution-proofs/final/<distro>/transcript.md`. Before kickoff, the agent must (a) replace `<owner>` in `docs/distribution-proofs/final/README.md` and the four final transcripts with the real GitHub org/user, and (b) confirm `https://github.com/<owner>/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage` returns 200. Then drive the operator through the full end-to-end loop on each distro per the per-distro Reproduction Notes already inlined in each `final/<distro>/transcript.md`: install libfuse2 (Debian/Ubuntu) / fuse-libs (Fedora) / fuse2 (CachyOS); `wget` the AppImage; `chmod +x`; run `--version`; launch full wizard end-to-end (CreateConfig → InstallModel → SetupUinput → ConfigurePtt); confirm main config screen; fire at least one stratagem by voice. Operator fills in the 8 fields (STATUS, DISTRO, KERNEL, APPIMAGE_VERSION, APPIMAGE_SIZE_BYTES, WIZARD_COMPLETED, STRATAGEM_FIRED, INSTALL_METHOD=appimage). Agent verifies all four final transcripts carry `STATUS: ok` and that `cargo test --test distribution_proofs -- --test-threads=1` still reports 11/11 passing (the same test file covers final transcripts via assert_final_transcript). If S04 has not shipped at the time this task is reached in auto-mode, the agent must record a structured blocker note in this task's section of the slice journal and pause — do NOT mark the task complete with placeholder transcripts. MEM094 reminder: libfuse2 (NOT libfuse3) on Debian/Ubuntu.

## Inputs

- `docs/distribution-proofs/final/debian13/transcript.md`
- `docs/distribution-proofs/final/ubuntu2604/transcript.md`
- `docs/distribution-proofs/final/fedora44/transcript.md`
- `docs/distribution-proofs/final/cachyos/transcript.md`
- `docs/distribution-proofs/final/README.md`
- `tests/distribution_proofs.rs`
- `docs/distribution-proofs/appimage/debian13/transcript.md`
- `docs/distribution-proofs/appimage/ubuntu2604/transcript.md`
- `docs/distribution-proofs/appimage/fedora44/transcript.md`
- `docs/distribution-proofs/appimage/cachyos/transcript.md`

## Expected Output

- `docs/distribution-proofs/final/debian13/transcript.md`
- `docs/distribution-proofs/final/ubuntu2604/transcript.md`
- `docs/distribution-proofs/final/fedora44/transcript.md`
- `docs/distribution-proofs/final/cachyos/transcript.md`
- `docs/distribution-proofs/final/README.md`

## Verification

All four final/<distro>/transcript.md files carry STATUS: ok with all 8 fields populated; `cargo test --test distribution_proofs -- --test-threads=1` passes 11/11 (covering both appimage and final transcript assertions). Verify with: `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/final/$d/transcript.md; done | grep -c '^STATUS: ok$'` returns 4, AND `cargo test --test distribution_proofs -- --test-threads=1 2>&1 | grep -E 'test result: ok\. 11 passed'` matches, AND `! grep -r '<owner>' docs/distribution-proofs/final/` exits 0 (placeholder fully resolved).

## Observability Impact

Signals added/changed: real APPIMAGE_VERSION, APPIMAGE_SIZE_BYTES, WIZARD_COMPLETED, STRATAGEM_FIRED replace `pending` placeholders, providing the milestone's final-assembly proof signal. How a future agent inspects this: the four final transcripts together encode whether the published AppImage works end-to-end on every supported distro. Failure state exposed: per-distro end-to-end failures surface as STATUS: failed:<reason> with the failed step recorded in the relevant field.
