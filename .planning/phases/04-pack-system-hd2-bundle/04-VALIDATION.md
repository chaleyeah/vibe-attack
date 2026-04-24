# Phase 4 Validation Strategy: Pack System + HD2 Bundle

## 1. Goal Verification (Goal-Backward)

| Goal | Success Criteria | Verification Method |
|------|------------------|---------------------|
| HD2 Bundle | 80+ stratagems covered | `grep -c "name:" profiles/hd2.yaml` >= 80 |
| Import/Export | Round-trip integrity | `hd-voice export hd2.hdpack && hd-voice import hd2.hdpack --name test && diff profiles/hd2.yaml profiles/test.yaml` |
| Profile Switching | Switch without restart | Send UDS command, verify `daemon` logs "Switched to profile: X" |
| TUI Editor | Persistence | Open `edit`, change a value, save, verify YAML file change |

## 2. Technical Verification (Nyquist Dimension 8)

### UDS Control Channel
- **Test**: `echo '{"cmd": "ping"}' | nc -U /run/user/$UID/hd-linux-voice.sock`
- **Expected**: `{"status": "pong"}`

### ZIP Path Traversal Protection
- **Test**: Create a malicious `.hdpack` with a file path `../../.ssh/authorized_keys`.
- **Expected**: Import fails with "Invalid path in pack" error.

### Flattening Logic
- **Test**: Unit test `Pack::flatten()` with nested categories.
- **Expected**: All macros present in a flat map, duplicates handled (last-writer-wins or error).

## 3. Manual Verification (UAT)

1. **First-run Flow**:
    - Delete `~/.config/hd-linux-voice`.
    - Run `hd-voice`.
    - Verify it offers to install the bundled HD2 pack.
2. **"The Tactical Test"**:
    - Open the TUI editor.
    - Change "Eagle Airstrike" phrase to "Bird Bomb".
    - Save and exit.
    - Speak "Bird Bomb" into the mic.
    - Verify the correct key sequence fires in-game.
