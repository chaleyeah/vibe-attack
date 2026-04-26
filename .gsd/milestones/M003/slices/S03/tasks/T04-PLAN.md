---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Implement ConfigurePtt panel with evdev capture thread

In wizard.rs, implement show_configure_ptt(ui, state, ptt_state). PttCaptureState holds: thread handle Option, captured_key Arc<Mutex<Option<String>>>, listening bool. On 'Listen for key' button click: spawn a thread that opens the first available evdev keyboard device, calls fetch_events() in a loop, on first KeyDown event converts the key to its name string (format!("{:?}", key)), stores it in captured_key, and exits. On next update() call, check captured_key and if Some: write it to config file's ptt.key field via a simple text replacement (read config, replace/append ptt.key line), call probe::run() to refresh. Show 'Listening... press any key' label while thread is running. Show captured key name when done.

## Inputs

- `src/input/ptt.rs (evdev usage pattern)`
- `config.example.yaml (ptt.key format)`

## Expected Output

- `PttCaptureState struct with Arc<Mutex<Option<String>>>`
- `Thread spawned on click, exits after first keydown`
- `Config file updated with KEY_* name`
- `probe::run() returns true for ptt check after write`

## Verification

Thread starts on button click; pressing a key stores the name; config file is updated; probe::run() returns ptt_configured=true after write
