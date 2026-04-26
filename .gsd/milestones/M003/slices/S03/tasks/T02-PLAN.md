---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Implement CreateConfig and InstallModel panels

In wizard.rs, implement show_create_config(ui, state) and show_install_model(ui, state). CreateConfig: heading 'Create config file', shows target path, a 'Copy example config' button that calls std::fs::copy then probe::run() to refresh. InstallModel: heading 'Install whisper model', shows model path, shows the curl command in a monospace code block (ui.monospace), a 'Re-check' button that calls probe::run(). Both panels show a success indicator when their check already passes.

## Inputs

- `config.example.yaml path (resolved from exe dir or env::current_dir at startup)`
- `src/ui/probe.rs`

## Expected Output

- `show_create_config renders heading, path, button`
- `show_install_model renders heading, monospace command, re-check button`

## Verification

Panels compile; CreateConfig button copies file when clicked (manual check)
