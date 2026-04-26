---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Load real profiles from XDG profiles dir

Add a load_profiles() function in src/ui/config_app.rs or a new src/ui/loader.rs. It should: resolve the XDG profiles directory via xdg::BaseDirectories::with_prefix('vibe-attack').get_config_home() and join 'profiles', read all *.yaml files, extract the profile name (stem of the filename), return Vec<String>. Log the count. In vibe-attack-config.rs, call load_profiles() after wizard completion and set ConfigApp.profiles. Re-load on 'Refresh' button click.

## Inputs

- `src/ui/probe.rs (XDG pattern)`
- `src/ui/config_app.rs`

## Expected Output

- `load_profiles() returns profile names from XDG profiles dir`
- `ConfigApp.profiles populated on startup and on Refresh click`
- `tracing::info logs count of profiles found`

## Verification

Place a test.yaml in ~/.config/vibe-attack/profiles/ and launch; profile list shows 'test'
