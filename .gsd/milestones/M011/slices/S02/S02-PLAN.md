# S02: VM proof runs — populate transcripts

**Goal:** Populate all 12 distribution-proof transcripts (4 distros × 3 proof trees: appimage, wizard, final) with real VM-run results so STATUS: ok appears in every file and the milestone proof gate passes.
**Demo:** all 12 transcripts (appimage + wizard + final × 4 distros) carry `STATUS: ok`; proof trees are complete.

## Must-Haves

- After this: all 12 transcripts carry `STATUS: ok` (none `pending VM run`); `cargo test --test distribution_proofs --test wizard_proofs -- --test-threads=1` reports 16/16 passing with the stricter ok-only acceptance recorded in the slice notes; any UI/wizard issues found during runs are filed against S03.

## Proof Level

- This slice proves: operational (real-runtime distribution behavior on four target distros). Real runtime required: yes — four VMs (Debian 13, Ubuntu 26.04, Fedora 44, CachyOS). Human/UAT required: yes — wizard scenarios A–D and end-to-end voice firing must be observed by a human.

## Integration Closure

Upstream surfaces consumed: `scripts/verify-appimage.sh`, `packaging/appimage/build.sh`, `target/release/vibe-attack-config`, the published GitHub Releases AppImage (T03 only). New wiring introduced in this slice: none — no code changes; only transcript content. What remains before the milestone is truly usable end-to-end: any UI fixes surfaced by wizard runs are deferred to S03; release CI lives in S04; final transcripts cannot reach `STATUS: ok` until S04 publishes a real AppImage at the GitHub Releases URL — T03 is gated on that prerequisite.

## Verification

- Runtime signals: each transcript carries STATUS, EXIT_CODE / SCENARIO_*, KERNEL, DISTRO; appimage transcripts also record SIZE_BYTES and SHA256 written by `scripts/verify-appimage.sh`. Inspection surfaces: `docs/distribution-proofs/{appimage,wizard,final}/<distro>/transcript.md` (one file per proof × distro); structural test commands surface failures by named test function. Failure visibility: on failure, transcripts carry `STATUS: failed:<reason>` and the relevant per-proof FAILURE_REASON / SCENARIO_* fields, preserved on disk even when the run aborts. Redaction constraints: none — transcripts are committed proof artifacts intended for public auditability.

## Tasks

- [x] **T01: Capture appimage proofs on all four distros and verify structural tests still pass** `est:human-bound; agent verification ~15m`
  Drive the human operator through running `bash packaging/appimage/build.sh && bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/<distro>/transcript.md` on each of Debian 13, Ubuntu 26.04, Fedora 44, and CachyOS VMs. The script writes all 7 transcript fields (STATUS, DISTRO, KERNEL, SIZE_BYTES, SHA256, EXIT_CODE, VERSION_OUTPUT) automatically, so the agent's role is (a) confirm the four `appimage/<distro>/transcript.md` files still pre-flight clean, (b) hand the operator the exact dependency-install + script command from `docs/distribution-proofs/appimage/README.md`, (c) verify the four resulting transcripts carry `STATUS: ok` and pass `cargo test --test distribution_proofs -- --test-threads=1`. If any distro returns `STATUS: failed:<reason>`, file a follow-up note in the slice's blocker log and surface the FAILURE_REASON field — do NOT downgrade the slice goal silently. Per-distro dependency commands are already correct in `docs/distribution-proofs/appimage/README.md` (apt-get for Debian/Ubuntu, dnf for Fedora 44, pacman for CachyOS). MEM094 reminder: Debian/Ubuntu need libfuse2 (NOT libfuse3). MEM081/MEM099 reminder: verify-appimage.sh always writes a transcript even on failure — failed runs leave inspectable proof. The script's 50 MB size guard (MEM081) means an oversize build trips `STATUS: failed:too-large` — confirm `--release` (the default in build.sh) is used.
  - Files: `docs/distribution-proofs/appimage/debian13/transcript.md`, `docs/distribution-proofs/appimage/ubuntu2604/transcript.md`, `docs/distribution-proofs/appimage/fedora44/transcript.md`, `docs/distribution-proofs/appimage/cachyos/transcript.md`, `docs/distribution-proofs/appimage/README.md`, `scripts/verify-appimage.sh`, `tests/distribution_proofs.rs`
  - Verify: All four appimage/<distro>/transcript.md files have STATUS: ok (or a documented STATUS: failed:<reason> with FAILURE_REASON), and `cargo test --test distribution_proofs -- --test-threads=1` passes 11/11 tests. Verify with: `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/appimage/$d/transcript.md; done | grep -c '^STATUS: ok$'` returns 4, AND `cargo test --test distribution_proofs -- --test-threads=1 2>&1 | grep -E 'test result: ok\. 11 passed'` matches.

- [ ] **T02: Run wizard UAT scenarios A–D on all four distros and record SCENARIO_* outcomes** `est:human-bound; agent verification ~20m`
  Drive the human operator through wizard UAT on each distro: build `vibe-attack-config` from source (`cargo build --release --bin vibe-attack-config`), then exercise the four documented scenarios from `docs/distribution-proofs/wizard/README.md` and the per-distro Reproduction Notes already inlined in each `wizard/<distro>/transcript.md`: A=fresh install with full wizard + voice firing, B=model-pre-placed (skip InstallModel), C=relaunch shows main config screen, D=`--skip-wizard` flag short-circuits. Each scenario's outcome (`ok` or `failed:<scenario-letter>`) is filled into the corresponding SCENARIO_A/B/C/D field; STRATAGEM_FIRED records voice-firing success from Scenario A. The agent's job is (a) before runs, confirm polkit agent presence is documented in the operator brief; (b) after runs, verify all four wizard transcripts carry `STATUS: ok` with all 10 fields populated, and that `cargo test --test wizard_proofs -- --test-threads=1` reports 5/5 passing. Pitfalls to surface to the operator: polkit dialog stalls on headless/Wayland VMs without an agent; `usermod -aG input` requires logout-relogin (or `newgrp input`) before Scenario A's mic test; HuggingFace 302 redirect on restricted networks — pre-place model and run Scenario B first to validate wizard logic independently of network. Real UI bugs surfaced during runs are NOT fixed in this task — file them as candidates for S03 by appending a `## Findings` block under Reproduction Notes if needed.
  - Files: `docs/distribution-proofs/wizard/debian13/transcript.md`, `docs/distribution-proofs/wizard/ubuntu2604/transcript.md`, `docs/distribution-proofs/wizard/fedora44/transcript.md`, `docs/distribution-proofs/wizard/cachyos/transcript.md`, `docs/distribution-proofs/wizard/README.md`, `tests/wizard_proofs.rs`
  - Verify: All four wizard/<distro>/transcript.md files carry STATUS: ok with SCENARIO_A/B/C/D = ok and STRATAGEM_FIRED populated; `cargo test --test wizard_proofs -- --test-threads=1` passes 5/5. Verify with: `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/wizard/$d/transcript.md; done | grep -c '^STATUS: ok$'` returns 4, AND `cargo test --test wizard_proofs -- --test-threads=1 2>&1 | grep -E 'test result: ok\. 5 passed'` matches.

- [ ] **T03: Run final end-to-end UAT on all four distros against the published AppImage and close the proof gate** `est:human-bound; agent verification ~20m; gated on S04 completion`
  PRECONDITION: this task is BLOCKED until S04 publishes a real AppImage to the GitHub Releases URL referenced by `docs/distribution-proofs/final/<distro>/transcript.md`. Before kickoff, the agent must (a) replace `<owner>` in `docs/distribution-proofs/final/README.md` and the four final transcripts with the real GitHub org/user, and (b) confirm `https://github.com/<owner>/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage` returns 200. Then drive the operator through the full end-to-end loop on each distro per the per-distro Reproduction Notes already inlined in each `final/<distro>/transcript.md`: install libfuse2 (Debian/Ubuntu) / fuse-libs (Fedora) / fuse2 (CachyOS); `wget` the AppImage; `chmod +x`; run `--version`; launch full wizard end-to-end (CreateConfig → InstallModel → SetupUinput → ConfigurePtt); confirm main config screen; fire at least one stratagem by voice. Operator fills in the 8 fields (STATUS, DISTRO, KERNEL, APPIMAGE_VERSION, APPIMAGE_SIZE_BYTES, WIZARD_COMPLETED, STRATAGEM_FIRED, INSTALL_METHOD=appimage). Agent verifies all four final transcripts carry `STATUS: ok` and that `cargo test --test distribution_proofs -- --test-threads=1` still reports 11/11 passing (the same test file covers final transcripts via assert_final_transcript). If S04 has not shipped at the time this task is reached in auto-mode, the agent must record a structured blocker note in this task's section of the slice journal and pause — do NOT mark the task complete with placeholder transcripts. MEM094 reminder: libfuse2 (NOT libfuse3) on Debian/Ubuntu.
  - Files: `docs/distribution-proofs/final/debian13/transcript.md`, `docs/distribution-proofs/final/ubuntu2604/transcript.md`, `docs/distribution-proofs/final/fedora44/transcript.md`, `docs/distribution-proofs/final/cachyos/transcript.md`, `docs/distribution-proofs/final/README.md`, `tests/distribution_proofs.rs`
  - Verify: All four final/<distro>/transcript.md files carry STATUS: ok with all 8 fields populated; `cargo test --test distribution_proofs -- --test-threads=1` passes 11/11 (covering both appimage and final transcript assertions). Verify with: `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/final/$d/transcript.md; done | grep -c '^STATUS: ok$'` returns 4, AND `cargo test --test distribution_proofs -- --test-threads=1 2>&1 | grep -E 'test result: ok\. 11 passed'` matches, AND `! grep -r '<owner>' docs/distribution-proofs/final/` exits 0 (placeholder fully resolved).

## Files Likely Touched

- docs/distribution-proofs/appimage/debian13/transcript.md
- docs/distribution-proofs/appimage/ubuntu2604/transcript.md
- docs/distribution-proofs/appimage/fedora44/transcript.md
- docs/distribution-proofs/appimage/cachyos/transcript.md
- docs/distribution-proofs/appimage/README.md
- scripts/verify-appimage.sh
- tests/distribution_proofs.rs
- docs/distribution-proofs/wizard/debian13/transcript.md
- docs/distribution-proofs/wizard/ubuntu2604/transcript.md
- docs/distribution-proofs/wizard/fedora44/transcript.md
- docs/distribution-proofs/wizard/cachyos/transcript.md
- docs/distribution-proofs/wizard/README.md
- tests/wizard_proofs.rs
- docs/distribution-proofs/final/debian13/transcript.md
- docs/distribution-proofs/final/ubuntu2604/transcript.md
- docs/distribution-proofs/final/fedora44/transcript.md
- docs/distribution-proofs/final/cachyos/transcript.md
- docs/distribution-proofs/final/README.md
