---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Implement download-model step

Add the install_model step: target path is $XDG_DATA_HOME/vibe-attack/models/whisper/ggml-tiny.en.bin (or ~/.local/share/vibe-attack/... if XDG_DATA_HOME unset). If file exists and size > 0, skip. Otherwise mkdir -p and download with curl -L showing progress. Print the URL and destination before downloading. If curl is not available, print a manual download instruction and exit 1.

## Inputs

- `config.example.yaml (for model_path reference)`
- `docs/troubleshooting.md (model download URL)`

## Expected Output

- `ggml-tiny.en.bin downloaded to XDG data path on first run`
- `Second run skips with file-exists message`
- `curl-absent path prints manual URL and exits 1`

## Verification

With XDG_DATA_HOME=$(mktemp -d) and network access: model downloads to correct path; re-run skips; with curl missing: prints manual instruction and exits 1
