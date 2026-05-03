# S03 Replan

**Milestone:** M013
**Slice:** S03
**Blocker Task:** T02
**Created:** 2026-05-03T21:52:14.722Z

## Blocker Description

T02 successfully pushed v1.0.1-test and confirmed CI passes, but the Release workflow failed on two structural incompatibilities: (1) RPM's Version field rejects the hyphen in `1.0.1-test` (`error: line 2: Illegal char '-' (0x2d) in: Version: 1.0.1-test`); RPM reserves `-` as the name-version-release separator. (2) `dpkg-buildpackage -uc -us -b` runs `dpkg-checkbuilddeps` which requires `cargo` and `rustc` as system packages, but the workflow installs Rust via rustup so dpkg sees them as missing (`Unmet build dependencies: cargo rustc`, exit 3). Both failures are in `.github/workflows/release.yml` and must be fixed before any retry can succeed. The stale `v1.0.1-test` tag and partial release on origin must also be cleaned up before the new test cycle.

## What Changed

Re-purposed T03 to fix the two structural blockers in `release.yml` (substitute `-`→`~` for RPM Version, add `-d` to dpkg-buildpackage to skip the build-dep precheck) and to clean up the stale `v1.0.1-test` tag and any partial GitHub Release left behind by the failed T02 run. Added T04 to push a fresh test tag (`v1.0.1-test2` to avoid any cache/state collision with the prior failure) and watch both CI + Release workflows to all-green. Added T05 to verify the release assets exist with correct version-stamped names — note that the RPM filename will contain `~` in place of `-` (i.e. `vibe-attack-1.0.1~test2-1.x86_64.rpm`) due to the new RPM tilde substitution — and then delete the test tag and release from origin and locally.
