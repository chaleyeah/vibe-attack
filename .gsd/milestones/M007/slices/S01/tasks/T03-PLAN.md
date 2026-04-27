---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Fix load_profiles to scan for {name}/pack.yaml subdirectories

In src/ui/config_app.rs, replace the existing load_profiles implementation that scans for flat profiles/*.yaml files with one that iterates entries of the profiles directory, treats each entry as a profile only if it is a directory containing a pack.yaml file, and uses the directory name as the profile name. The format must match what Pack::load_from_dir and handle_switch_profile already use.

## Inputs

- `src/ui/config_app.rs current load_profiles implementation that scans profiles/*.yaml flat files`

## Expected Output

- `load_profiles returns Vec<String> of profile names (directory names) where each named directory contains pack.yaml`

## Verification

Code review: load_profiles iterates read_dir, filters DirEntry::file_type().is_dir() && entry.path().join("pack.yaml").exists(); flat .yaml files are NOT included in the returned list; cargo check passes
