---
phase: M011
phase_name: v1.0 Release
project: hd-linux-voice
generated: 2026-04-29T12:00:00Z
counts:
  decisions: 10
  lessons: 9
  patterns: 6
  surprises: 4
missing_artifacts: []
---

# M011 Learnings

### Decisions

- **4-job symmetric release workflow.** Chose a 3-parallel-build + 1-release-collector architecture over a single monolithic job. Each build job emits `upload-artifact`; the release job collects all three via `download-artifact`. Independently retryable and extensible to new package types.
  Source: S04-SUMMARY.md/Key decisions

- **rpmbuild --nodeps on Ubuntu CI hosts.** Chose `--nodeps` over switching to a Fedora container. The workflow apt-get step pre-installs all real build deps; `--nodeps` only skips rpm's internal dependency resolution which rejects Fedora-style BuildRequires on apt hosts.
  Source: S05-SUMMARY.md/Key decisions

- **LD_LIBRARY_PATH (not --library flags) for linuxdeploy dlopen resolution.** linuxdeploy's `--library` flag approach was tried first and failed. `libsherpa-onnx-c-api.so` is loaded via dlopen at runtime, not ELF RPATH — ldd cannot find it; setting `LD_LIBRARY_PATH=AppDir/usr/lib` before invoking linuxdeploy is the correct fix.
  Source: S05-SUMMARY.md/What Happened

- **find_so() fallback to target/sherpa-onnx-prebuilt/ for Rust cache hits.** When the Rust build cache is a full hit, `cargo build --release` is a no-op; build.rs and ort crate scripts never run and never copy .so files to `target/release/`. The AppImage build script must check `target/sherpa-onnx-prebuilt/` as a fallback path.
  Source: S05-SUMMARY.md/What Happened

- **egui PttCaptureState field (not frame-local variable) for PTT key capture.** egui repaints on every frame; frame-local variables reset each repaint. Stateful captures (TextEdit state, PTT key input) must be stored in a persistent struct field, not a frame-local variable.
  Source: S03-SUMMARY.md/Key decisions

- **Arc<AtomicBool> + take pattern for tray Quit signal.** Chosen over process::exit in ksni D-Bus callbacks. The take pattern (swap false, AcqRel) mirrors the open_window flag pattern; the eframe main loop polls it each tick and exits cleanly.
  Source: S03-SUMMARY.md/Key decisions

- **tooltip_description_for as free pub(crate) function.** Extracted state-derived display logic out of ksni callback closures so it can be unit-tested without D-Bus or ksni instantiation. Follows MEM045 convention from M008.
  Source: S03-SUMMARY.md/Key decisions

- **probe::run() called per-frame in DownloadStatus::Done arm.** Idempotent filesystem read; calling it per-frame enables auto-advance on wizard re-entry without adding a separate trigger mechanism.
  Source: S03-SUMMARY.md/Key decisions

- **v1.0.0 tag immutable post-publish.** Force-moving a published tag would invalidate PKGBUILD sha256sums and break any AUR installs that have already cached the digest. Tags must be treated as immutable once published.
  Source: S05-SUMMARY.md/Key decisions

- **permissions: contents: write required explicitly on release job.** GITHUB_TOKEN defaults to read-only contents scope. Any job that calls `gh release create` or uploads release assets needs an explicit `permissions: contents: write` stanza.
  Source: S05-SUMMARY.md/What Happened

### Lessons

- **GITHUB_TOKEN is read-only by default — release jobs need explicit write permission.** The release job received a 403 on run 3. Root cause: `permissions: contents: write` must be declared on the job (or workflow) that publishes release assets. This is a silent footgun in GitHub Actions.
  Source: S05-SUMMARY.md/What Happened

- **Rust build cache hits suppress build.rs — AppImage .so discovery must have a prebuilt fallback.** A full Rust cache hit means `cargo build --release` exits immediately; no build.rs runs; the ort crate never copies `libonnxruntime.so` to `target/release/`. Any script that assumes `target/release/*.so` will exist must also check `target/sherpa-onnx-prebuilt/`.
  Source: S05-SUMMARY.md/What Happened

- **linuxdeploy cannot discover dlopen-only .so files — LD_LIBRARY_PATH is required.** linuxdeploy uses ldd to find transitive dependencies. dlopen-only libraries (like sherpa-onnx C API) don't appear in ELF RPATH and are invisible to ldd. Only LD_LIBRARY_PATH forces dlopen to find them.
  Source: S05-SUMMARY.md/What Happened

- **debian/compat and debhelper-compat in Build-Depends are mutually exclusive.** Having both causes `dh: error: debhelper compat level specified both in debian/compat and via build-dependency on debhelper-compat`. Delete the compat file; use only the modern build-dep form.
  Source: S05-SUMMARY.md/What Happened

- **RPM %install + %files both handling the same file causes an "unpackaged file" error.** If both `%install` (manual `install -Dm644`) and `%files` (`%doc README.md`) claim a file, rpmbuild fails with "Installed (but unpackaged) file(s) found." Remove the manual install from `%install` and let `%doc` handle it.
  Source: S05-SUMMARY.md/What Happened

- **Wizard UAT cannot be automated in auto-mode.** Polkit + desktop GUI session with human observation is required for wizard scenario coverage. Structural tests (field presence, valid STATUS values) are the only CI-verifiable signal; STATUS: ok closure is operator-bound.
  Source: S02-SUMMARY.md/Known limitations

- **Structural distribution proof tests with STATUS: pending VM run keep CI green during incremental transcript population.** VALID_STATUSES includes 'pending VM run', so tests pass before VM runs complete. This allows parallel human/agent work without blocking CI.
  Source: S01-SUMMARY.md/Known limitations

- **egui repaints every frame — stateful UI must live in persistent structs.** Any state that must survive between frames (TextEdit buffers, key captures, download progress) must be stored in a struct field, not a local variable inside the UI closure.
  Source: S03-SUMMARY.md/Key decisions

- **Static YAML contract tests in packaging.rs catch CI job structural regressions at cargo test time.** String-contains checks on raw .yml content (job names, cache key counts, artifact globs) enforce CI structure without requiring a real tag push. This provides fast feedback on release.yml changes.
  Source: S04-SUMMARY.md/Patterns established

### Patterns

- **4-job release pipeline: 3 parallel build jobs → 1 release collector.** Each build job emits `upload-artifact`; the release job uses `needs: [build-appimage, build-deb, build-rpm]` and `download-artifact` to collect all artifacts before publishing. `fail_on_unmatched_files: true` provides loud failure on missing globs.
  Source: S04-SUMMARY.md/Patterns established

- **AppImage .so discovery: target/release/ first, target/sherpa-onnx-prebuilt/ fallback.** The find_so() function in build.sh searches target/release/ (normal build) then falls back to the sherpa-onnx prebuilt cache hierarchy (Rust cache hit scenario). Both paths must be searched.
  Source: S05-SUMMARY.md/Patterns established

- **CI cross-distro packaging on Ubuntu: --nodeps for RPM, symlinked debian/ for .deb.** Ubuntu CI hosts can build both RPM and Debian packages by pre-installing all deps via apt-get and using package-format-specific workarounds rather than per-distro containers.
  Source: S05-SUMMARY.md/Patterns established

- **Pending-VM-run transcript pattern with structural validation.** Scaffolded transcripts carry STATUS: pending VM run. Structural tests accept this value, keeping CI green. Human operator updates STATUS to ok after running VM sessions. This separates structural correctness (auto-verifiable) from runtime correctness (human-bound).
  Source: S02-SUMMARY.md/Patterns established

- **Per-distro Reproduction Notes inlined in each transcript.md.** Each proof transcript includes self-contained operator instructions as Reproduction Notes — no external runbook lookup needed. Failed runs leave STATUS: failed:<reason> artifacts for post-mortem.
  Source: S02-SUMMARY.md/Patterns established

- **Free pub(crate) functions for state-derived display logic (icon, tooltip).** Extracting tray icon and tooltip computation out of ksni callbacks into free functions enables unit tests without D-Bus. Pattern established in M008 (icon_name_for_state) and extended in M011 (tooltip_description_for).
  Source: S03-SUMMARY.md/Patterns established

### Surprises

- **Five CI defects surfaced on the first real tag push.** The release pipeline had never been exercised end-to-end. Static packaging tests validated structure but not runtime behavior. Four workflow iterations were needed to clear all five defects: rpmbuild --nodeps, LD_LIBRARY_PATH, debian/compat conflict, rpm %install/%files conflict, GITHUB_TOKEN permissions.
  Source: S05-SUMMARY.md/What Happened

- **Rust cache hit caused build-appimage to fail on run 4 after three other jobs already succeeded.** The Rust build cache hit on run 4 was a full hit, causing cargo build --release to be a no-op and leaving no .so files in target/release/. This was not anticipated in the S04 plan; find_so() fallback was added reactively.
  Source: S05-SUMMARY.md/What Happened

- **unauthenticated GitHub release URL verification not applicable for private repos.** The S05 T01 plan included a `curl -I` HTTP 200 check against the release download URL. Private repos return 404 for unauthenticated requests even on published releases. gh API verification was used instead.
  Source: S05-SUMMARY.md/Deviations

- **S03 UI polish work had no empirical wizard-run findings to act on.** S02 only captured one real AppImage run (ubuntu2604); all wizard transcripts remained STATUS: pending VM run. S03 fixed pre-existing bugs found via code review rather than VM-run-driven triage. The planned pipeline (VM runs → findings → polish) partially inverted: polish preceded VM evidence.
  Source: S03-SUMMARY.md/Follow-ups
