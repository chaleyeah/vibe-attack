# S02: VM proof runs — populate transcripts — Research

**Date:** 2026-04-28

## Summary

S02 is a **human-executed, agent-scaffolded** slice. The agent's role is to verify the scaffold is complete and correct, then plan any remaining scaffold tasks. The human's role is to boot four VMs and run the proof scripts, then commit updated transcripts.

All 12 transcript files already exist at the correct paths with `STATUS: pending VM run` placeholders and full reproduction notes. All three test harnesses (`distribution_proofs.rs`, `wizard_proofs.rs`) already accept `pending VM run` as a valid status — they test structural completeness only. The tests are already passing (confirmed in S01 verification). There is no code to write and no structural scaffolding missing.

The slice reduces to: **run the VMs, fill in the transcripts, verify the tests still pass.** The planner should decompose this into one task per proof tree (appimage / wizard / final), each of which is a human-action task with agent-side verification that the resulting transcripts pass the test suite.

## Recommendation

Plan three tasks:
1. **T01 — Run appimage proof on all 4 distros** — human executes `bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/<distro>/transcript.md` on each VM; agent verifies `cargo test --test distribution_proofs -- --test-threads=1` passes with `STATUS: ok` in all four transcripts.
2. **T02 — Run wizard UAT on all 4 distros** — human executes the four wizard scenarios (A–D) per the reproduction notes in each transcript; agent verifies `cargo test --test wizard_proofs -- --test-threads=1` passes.
3. **T03 — Run final UAT on all 4 distros** — human executes the full end-to-end loop (AppImage download → wizard → stratagem fires) per the reproduction notes; agent verifies `cargo test --test distribution_proofs -- --test-threads=1` still passes for final transcripts.

No code changes are expected unless proof runs surface failures — in that case follow-on UI polish is owned by S03.

## Implementation Landscape

### Key Files

- `docs/distribution-proofs/appimage/{debian13,ubuntu2604,fedora44,cachyos}/transcript.md` — 7-field transcripts; `STATUS: pending VM run`; reproduction notes tell the human exactly what to run; `verify-appimage.sh` writes all fields automatically
- `docs/distribution-proofs/wizard/{debian13,ubuntu2604,fedora44,cachyos}/transcript.md` — 10-field transcripts; `STATUS: pending VM run`; four UAT scenarios (A–D) documented inline; human fills fields manually after observing each scenario
- `docs/distribution-proofs/final/{debian13,ubuntu2604,fedora44,cachyos}/transcript.md` — 8-field transcripts; `STATUS: pending VM run`; human runs the full end-to-end AppImage loop and fills fields manually
- `scripts/verify-appimage.sh` — automates the appimage proof; writes all 7 transcript fields unconditionally (even on failure); accepts a path argument; already handles `skipped:tools-missing` gracefully
- `tests/distribution_proofs.rs` — validates appimage (7-field) and final (8-field) transcripts; accepts `ok`, `skipped:tools-missing`, `pending VM run`, and `failed:<reason>` statuses; will require `STATUS: ok` in all 8 transcripts to pass the milestone gate
- `tests/wizard_proofs.rs` — validates wizard (10-field) transcripts; accepts `ok`, `pending VM run`, and `failed:scenario-*` statuses; will require `STATUS: ok` in all 4 wizard transcripts to pass the milestone gate
- `docs/distribution-proofs/appimage/README.md` — per-distro reproduction commands (copy-pasteable); dependency installs per package manager
- `docs/distribution-proofs/wizard/README.md` — scenario A–D step-by-step guide + common pitfalls (polkit agent, input group re-login, HuggingFace redirect, evdev device selection)
- `docs/distribution-proofs/final/README.md` — final UAT loop guide + field capture commands

### Transcript Field Summary

**Appimage** (auto-written by `verify-appimage.sh`): `STATUS`, `DISTRO`, `KERNEL`, `SIZE_BYTES`, `SHA256`, `EXIT_CODE`, `VERSION_OUTPUT`

**Wizard** (manually filled): `STATUS`, `DISTRO`, `KERNEL`, `BINARY` (fixed: `vibe-attack-config`), `BINARY_VERSION`, `SCENARIO_A`, `SCENARIO_B`, `SCENARIO_C`, `SCENARIO_D`, `STRATAGEM_FIRED`

**Final** (manually filled): `STATUS`, `DISTRO`, `KERNEL`, `APPIMAGE_VERSION`, `APPIMAGE_SIZE_BYTES`, `WIZARD_COMPLETED`, `STRATAGEM_FIRED`, `INSTALL_METHOD` (fixed: `appimage`)

### Build Order

1. **Appimage proofs first** — they are fully automated (script writes transcript), lowest friction, and confirm the build pipeline works on each distro. Failures here (build errors, missing deps, wrong package names) surface early and block nothing downstream.
2. **Wizard proofs second** — requires a desktop session + polkit agent; more complex; four scenarios per distro. Pitfalls are documented in `wizard/README.md`. Run after appimage is confirmed so the binary is known-good.
3. **Final proofs last** — requires a released AppImage (not yet published), so in practice final proofs are blocked on S04 (version bump + release CI). The plan should note this dependency: final proofs can only be done after a real AppImage artifact exists at the GitHub Releases URL. The transcript placeholder notes use `https://github.com/<owner>/vibe-attack/releases/latest/download/...` — the `<owner>` token must be resolved before the final run.

### Verification Approach

After each batch of transcript updates:

```bash
# Appimage + final structural tests
cargo test --test distribution_proofs -- --test-threads=1

# Wizard structural tests
cargo test --test wizard_proofs -- --test-threads=1
```

Milestone gate: all 16 tests must pass with `STATUS: ok` in every transcript (none `pending VM run`) before S02 can be marked complete.

## Constraints

- `--test-threads=1` is required for distribution_proofs and wizard_proofs tests (pre-existing flake documented in S01).
- Final proofs depend on a published AppImage artifact — in practice, final proof transcripts can only be filled in after S04 (release CI) produces an artifact. The slice will need to be sequenced accordingly.
- The `final/README.md` uses `https://github.com/<owner>/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage` — the `<owner>` token needs to be replaced with the real GitHub org/user before the human runs the final proof. The planner should note this as a pre-condition for T03.
- CachyOS uses `pacman`, Debian 13 and Ubuntu 26.04 use `apt-get`, Fedora 44 uses `dnf` — dependency installs differ per distro (already correct in the READMEs).

## Common Pitfalls

- **polkit agent required on Wayland** — wizard Scenario A step 3 calls `pkexec` for `modprobe uinput`; on a headless or Wayland VM without a polkit agent the dialog silently stalls. Verify `ps aux | grep polkit` before starting wizard runs.
- **`input` group re-login required** — `usermod -aG input $USER` does not take effect in the current session. After Scenario A step 3, log out and back in (or `newgrp input`) before Scenario C.
- **Final proof blocked on real artifact** — `final/transcript.md` Reproduction Notes reference the GitHub Releases URL, which requires a published AppImage. Do not attempt final runs until S04 completes.
- **HuggingFace CDN redirect** — wizard model download step uses `ureq` to follow a 302 redirect. On restricted-network VMs, pre-download the model and use Scenario B (model pre-placed) path first to confirm wizard logic is independent of network.
- **AppImage FUSE2 on Fedora 44** — `fuse-libs` is the correct package (not `fuse2` which is the Arch/CachyOS name); the README already has the right `dnf` command.
- **`verify-appimage.sh` size guard** — the script rejects AppImages > 50 MB. If the build includes extra debug symbols the size check will fail with `STATUS: failed:too-large`. Build with `--release` (build.sh already does this).

## Open Risks

- **Final proofs blocked on S04** — S02 technically cannot fully close until the final/transcript.md files carry `STATUS: ok`, but those require a real AppImage artifact from S04. The slice may need to be split or the final proof tasks deferred to after S04.
- **Distro availability** — Debian 13 (Trixie) and Ubuntu 26.04 may not have stable ISO images yet as of the run date; verify download availability before scheduling VM setup time.
- **Fedora 44 / CachyOS package names** — package names were scaffolded from known conventions; actual package availability on the target distro versions should be confirmed at VM setup time.
