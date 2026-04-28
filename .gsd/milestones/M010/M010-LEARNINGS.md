---
phase: M010
phase_name: "Distribution — AppImage, AUR, First-Run Wizard"
project: hd-linux-voice
generated: "2026-04-28T11:30:00Z"
counts:
  decisions: 7
  lessons: 6
  patterns: 5
  surprises: 3
missing_artifacts: []
---

### Decisions

- **STATUS: skipped:tools-missing (exit 0) for verify-appimage.sh**: When linuxdeploy/appimagetool are absent, the script exits 0 with STATUS: skipped:tools-missing and writes a partial transcript including FAILURE_REASON. CI structure tests pass; execution completeness is deferred to VM runs. Chose this over exit 1 because the harness itself is the deliverable at CI time — the transcript format and reproduction instructions are what need to be correct.
  Source: S01-SUMMARY.md/Key decisions

- **Transcript written unconditionally even on failure**: Partial proof stays inspectable with STATUS: failed:<reason>. Chose unconditional write over skip-on-failure so the next human operator always has a file to examine regardless of which step failed.
  Source: S01-SUMMARY.md/Key decisions

- **Pending-VM-run transcript pattern (MEM079)**: All transcript fields present as 'pending' placeholders plus reproduction instructions; structural CI tests pass before real VM runs. Chose this over blocking CI on VM availability because VM runs require hardware and human operator time that cannot be automated.
  Source: S01-SUMMARY.md/Patterns established

- **onnxruntime in PKGBUILD depends (not makedepends)**: RPATH=$ORIGIN resolves in AppImage (co-located .so files) but not in Arch native install where only binaries land in /usr/bin/; system onnxruntime package required at runtime. Alternative of bundling onnxruntime in the package was rejected due to license/packaging complexity.
  Source: S04-SUMMARY.md/Key decisions

- **SHERPA_ONNX_ARCHIVE_DIR=$srcdir escape hatch**: Add prebuilt archive as source[] entry and export this env var in build() to prevent sherpa-onnx-sys from attempting network downloads inside the makepkg sandbox. This is the documented upstream escape hatch for offline Arch builds.
  Source: S04-SUMMARY.md/Key decisions

- **zip -j (junk-paths) for hdpack**: pack.yaml lands at archive root rather than nested under profiles/hd2/. Chose junk-paths so the consumer can unzip and find pack.yaml directly without path traversal logic.
  Source: S03-SUMMARY.md/Key decisions

- **Separate REQUIRED_FIELDS constants per proof level**: assert_transcript for AppImage (7 fields), assert_wizard_transcript for wizard (10 fields), assert_final_transcript for final UAT (8 fields) — each level independently extensible without coupling. Alternative of a single shared constant was rejected because the field sets differ by design.
  Source: S06-SUMMARY.md/Key decisions

### Lessons

- **PKGBUILD makedepends require cross-checking against Debian/RPM build deps**: bindgen/clang-sys transitive dependencies of sherpa-onnx-sys do not appear in Cargo.toml directly, so they are easy to miss. clang was missing from makedepends until S04. Cross-checking against Debian Build-Depends and RPM BuildRequires catches these gaps.
  Source: S04-SUMMARY.md/Key decisions

- **Binary spawn tests in Rust require LD_LIBRARY_PATH guard**: When spawning the built binary from integration tests (env!(CARGO_BIN_EXE_...)), LD_LIBRARY_PATH=target/debug/ must be set explicitly so the binary finds .so files at runtime. The rpath-relative resolution that works in production does not apply during test execution.
  Source: S02-SUMMARY.md/Key decisions

- **AppImage and AUR are different RPATH worlds**: AppImage co-locates .so files so $ORIGIN rpath works without system packages. Arch native installs only put binaries in /usr/bin/ so runtime .so resolution requires system packages. This distinction must be explicit in packaging docs.
  Source: S04-SUMMARY.md/Key decisions

- **Copying sherpa cache block verbatim from ci.yml to release.yml is correct**: Using a reusable workflow would require maintaining the reusable workflow's cache keys in sync; verbatim copy maintains key/path/conditional parity with zero merge risk. The intentional duplication is better than DRY indirection that could silently diverge.
  Source: S03-SUMMARY.md/Key decisions

- **The .desktop Exec= field must exactly match the binary name**: The initial .desktop had Exec=vibe-attack (the daemon) instead of Exec=vibe-attack-config (the GUI). Test assertion uses exact match (not substring) to catch this class of silent regression early.
  Source: S02-SUMMARY.md/Key decisions

- **assert Exec=value exactly, not as substring**: Substring assertions on .desktop fields allow typos and partial matches to pass silently. Exact equality is the correct assertion for fields with a single expected value.
  Source: S02-SUMMARY.md/Patterns established

### Patterns

- **Distribution proof transcript format**: 7-field minimum (STATUS, DISTRO, KERNEL, SIZE_BYTES, SHA256, EXIT_CODE, VERSION_OUTPUT) written unconditionally; STATUS line is the health signal. Pending-VM-run transcripts use 'pending' as placeholder for all fields except statically-known ones (e.g. INSTALL_METHOD: appimage). Tests assert all required fields are present regardless of STATUS value.
  Source: S01-SUMMARY.md/Patterns established

- **Proof directory scaffold pattern (MEM079)**: Seed transcripts with STATUS: pending VM run and structural tests at slice time; human operators convert to ok after real VM runs. Applied consistently across appimage/, wizard/, and final/ proof levels. This allows CI to gate on structure without requiring VM hardware.
  Source: S06-SUMMARY.md/Patterns established

- **Offline Arch builds via SHERPA_ONNX_ARCHIVE_DIR**: Add prebuilt archive as source[] entry in PKGBUILD; export SHERPA_ONNX_ARCHIVE_DIR=$srcdir in build() to bypass cargo-time network calls. Generalizes to any sherpa-onnx-sys consumer in an airgapped build environment.
  Source: S04-SUMMARY.md/Patterns established

- **Packaging cross-check pattern**: Compare makedepends across Debian Build-Depends, RPM spec BuildRequires, and PKGBUILD makedepends before finalizing any packaging format. Catches transitive build deps that are absent from Cargo.toml but required at compile time (e.g. clang for bindgen).
  Source: S04-SUMMARY.md/Patterns established

- **Release workflow artifact contract tests**: Structural assertions in tests/packaging.rs verify the release.yml YAML references correct artifact globs (with fail_on_unmatched_files: true) before any tag push. This catches broken upload steps at PR-review time instead of at release time when artifacts are expected.
  Source: S03-SUMMARY.md/Patterns established

### Surprises

- **clippy not available in the project's system cargo environment**: /usr/bin/cargo does not have the clippy component installed (no rustup on this host). The `cargo clippy -D warnings clean` success criterion could not be mechanically verified at milestone close. cargo check passes clean; all 27 test suites pass. This is an environment gap to address before the first release.
  Source: M010-ROADMAP.md/Success Criteria (verified at milestone close)

- **linuxdeploy and appimagetool are absent on the current build host (Ubuntu 26.04)**: Despite Ubuntu being Debian-derived and meeting the Debian 12 distro target in spirit, the AppImage packaging tools are not pre-installed. The verify-appimage.sh STATUS: skipped:tools-missing design absorbed this gracefully, but it means no AppImage binary has been produced yet — only the CI workflow to produce one.
  Source: S01-SUMMARY.md/Known limitations

- **AUR sha256sums must be SKIP placeholders in the repo**: The real hashes can only be computed after the release tarball exists (which requires the tag push). This means the PKGBUILD in-repo always has SKIP placeholders; the release workflow or human maintainer pins them at tag time. This is normal AUR practice but was not obvious at the start of S04.
  Source: S04-SUMMARY.md/Known limitations
