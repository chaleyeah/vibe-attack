---
id: T03
parent: S01
milestone: M003
key_files:
  - scripts/setup.sh
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-26T00:10:48.131Z
blocker_discovered: false
---

# T03: Implemented step_install_model: XDG data path, size check, curl download with progress, curl-absent fallback

**Implemented step_install_model: XDG data path, size check, curl download with progress, curl-absent fallback**

## What Happened

step_install_model targets $XDG_DATA_HOME/vibe-attack/models/whisper/ggml-tiny.en.bin. Skips if file exists and is non-empty (-s check). Detects curl absence and prints manual download instructions then exits 1. Otherwise prompts (respecting --yes) and downloads with curl -L --progress-bar.

## Verification

Re-run against temp dir where model exists shows skip; curl-absent path verified by renaming curl in test

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `XDG_DATA_HOME=dir_with_model bash scripts/setup.sh --yes --step=install_model` | 0 | pass — skip message printed | 25ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `scripts/setup.sh`
