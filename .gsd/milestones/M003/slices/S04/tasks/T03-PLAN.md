---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Add log channel feed to main config view

Add a log mpsc channel to VibeAttackConfigApp: sender side is passed to vibe-attack-config.rs as a simple fn add_log(msg). In the update() loop, drain all pending messages from the receiver into ConfigApp.log_lines (respecting MAX_LOG_LINES cap). Add a tracing subscriber layer that writes to the channel (using a custom Layer from tracing_subscriber) so daemon log events appear in the UI. ScrollArea auto-scrolls to bottom when new lines arrive (track last_line_count, scroll when it changes).

## Inputs

- `src/ui/config_app.rs (add_log_line, MAX_LOG_LINES)`
- `src/bin/vibe-attack-config.rs`

## Expected Output

- `mpsc channel receiver drained each frame`
- `tracing subscriber layer sends formatted log strings to channel`
- `ScrollArea auto-scrolls when new lines arrive`

## Verification

Launch app: log lines appear in the scrollable area; each probe::run() call generates a visible log line; area auto-scrolls on new lines
