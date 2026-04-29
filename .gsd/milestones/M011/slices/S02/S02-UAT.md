# S02: VM proof runs — populate transcripts — UAT

**Milestone:** M011
**Written:** 2026-04-29T01:33:10.650Z

# S02 UAT: VM Proof Run Verification

## Preconditions

- Four target VMs available: Debian 13, Ubuntu 26.04 LTS, Fedora 44, CachyOS
- For wizard UAT: full desktop session with polkit agent running (`ps aux | grep polkit`)
- For final UAT: S04 has published a GitHub Release with `vibe-attack-x86_64.AppImage` asset
- Repo cloned and `cargo build --release` completed on each distro VM
- For Debian/Ubuntu: `sudo apt-get install -y libfuse2 librsvg2-bin`
- For Fedora: `sudo dnf install -y fuse-libs librsvg2-tools`
- For CachyOS/Arch: `sudo pacman -Sy --noconfirm fuse2 librsvg`

---

## TC-01: AppImage proof — Debian 13

**Steps:**
1. On Debian 13 VM: `cd /path/to/repo`
2. `LD_LIBRARY_PATH=$(pwd)/target/release bash packaging/appimage/build.sh`
3. `bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/debian13/transcript.md`
4. `head -1 docs/distribution-proofs/appimage/debian13/transcript.md`

**Expected:** `STATUS: ok` on line 1; SIZE_BYTES < 52428800; SHA256 populated; EXIT_CODE: 0; VERSION_OUTPUT contains `vibe-attack`

---

## TC-02: AppImage proof — Fedora 44

**Steps:**
1. On Fedora 44 VM: `cd /path/to/repo`
2. `LD_LIBRARY_PATH=$(pwd)/target/release bash packaging/appimage/build.sh`
3. `bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/fedora44/transcript.md`
4. `head -1 docs/distribution-proofs/appimage/fedora44/transcript.md`

**Expected:** Same as TC-01 with Fedora 44 DISTRO/KERNEL values

---

## TC-03: AppImage proof — CachyOS

**Steps:**
1. On CachyOS VM: `cd /path/to/repo`
2. `LD_LIBRARY_PATH=$(pwd)/target/release bash packaging/appimage/build.sh`
3. `bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/cachyos/transcript.md`
4. `head -1 docs/distribution-proofs/appimage/cachyos/transcript.md`

**Expected:** Same as TC-01 with CachyOS DISTRO/KERNEL values

---

## TC-04: Distribution proofs structural test suite

**Steps (any host with all 4 appimage transcripts populated):**
1. `cargo test --test distribution_proofs -- --test-threads=1 2>&1 | grep 'test result'`
2. `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/appimage/$d/transcript.md; done | grep -c '^STATUS: ok$'`

**Expected:** `test result: ok. 11 passed`; grep count returns 4

---

## TC-05: Wizard UAT — Ubuntu 26.04 (all four scenarios)

**Steps:**
1. On Ubuntu 26.04 VM with desktop session:
2. `cargo build --release --bin vibe-attack-config`
3. **Scenario A** (fresh install): `LD_LIBRARY_PATH=target/release target/release/vibe-attack-config` → complete CreateConfig → InstallModel → SetupUinput → ConfigurePtt → fire one stratagem by voice
4. **Scenario B** (model pre-placed): pre-place model file, rerun → confirm InstallModel step is skipped
5. **Scenario C** (relaunch): relaunch after Scenario A → confirm wizard does NOT appear, main config screen shown
6. **Scenario D** (`--skip-wizard`): `LD_LIBRARY_PATH=target/release target/release/vibe-attack-config --skip-wizard` → confirm main config screen appears directly
7. Fill in all 10 fields in `docs/distribution-proofs/wizard/ubuntu2604/transcript.md`

**Expected:** STATUS: ok; SCENARIO_A=ok; SCENARIO_B=ok; SCENARIO_C=ok; SCENARIO_D=ok; STRATAGEM_FIRED=yes

**Pitfalls:** polkit agent must be running; `usermod -aG input $USER` requires `newgrp input` or re-login before mic test; pre-place model for Scenario B if HuggingFace CDN is restricted

---

## TC-06: Wizard UAT — Debian 13 (all four scenarios)

Same steps as TC-05 on Debian 13 VM. Fill `docs/distribution-proofs/wizard/debian13/transcript.md`.

---

## TC-07: Wizard UAT — Fedora 44 (all four scenarios)

Same steps as TC-05 on Fedora 44 VM. Fill `docs/distribution-proofs/wizard/fedora44/transcript.md`.

---

## TC-08: Wizard UAT — CachyOS (all four scenarios)

Same steps as TC-05 on CachyOS VM. Fill `docs/distribution-proofs/wizard/cachyos/transcript.md`.

---

## TC-09: Wizard proofs structural test suite

**Steps (after all 4 wizard transcripts populated):**
1. `cargo test --test wizard_proofs -- --test-threads=1 2>&1 | grep 'test result'`
2. `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/wizard/$d/transcript.md; done | grep -c '^STATUS: ok$'`

**Expected:** `test result: ok. 5 passed`; grep count returns 4

---

## TC-10: Final end-to-end UAT — all four distros (requires S04 release)

**Precondition:** `curl -s -o /dev/null -w "%{http_code}" https://github.com/chaleyeah/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage` returns 200

**Steps (per distro):**
1. Install FUSE2: `sudo apt-get install -y libfuse2` (Debian/Ubuntu) / `sudo dnf install -y fuse-libs` (Fedora) / `sudo pacman -Sy --noconfirm fuse2` (CachyOS)
2. `wget https://github.com/chaleyeah/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage`
3. `chmod +x vibe-attack-x86_64.AppImage`
4. `./vibe-attack-x86_64.AppImage --version` — confirm output and exit 0
5. Run full wizard end-to-end: CreateConfig → InstallModel → SetupUinput → ConfigurePtt
6. Confirm main config screen appears
7. Fire at least one stratagem by voice
8. Fill in 8 fields in `docs/distribution-proofs/final/<distro>/transcript.md`

**Expected:** STATUS: ok; WIZARD_COMPLETED=yes; STRATAGEM_FIRED=yes; INSTALL_METHOD=appimage; APPIMAGE_VERSION matches release tag

---

## TC-11: Full proof gate — all 12 transcripts STATUS: ok

**Steps (after all VM runs complete):**
1. `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/appimage/$d/transcript.md; done | grep -c '^STATUS: ok$'` → 4
2. `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/wizard/$d/transcript.md; done | grep -c '^STATUS: ok$'` → 4
3. `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/final/$d/transcript.md; done | grep -c '^STATUS: ok$'` → 4
4. `cargo test --test distribution_proofs --test wizard_proofs -- --test-threads=1 2>&1 | grep 'test result'` → 16/16 passed

**Expected:** All three grep counts return 4; test result: ok. 16 passed — slice S02 proof gate closed
