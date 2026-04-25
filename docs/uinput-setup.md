# uinput Setup Guide

The `vibe-attack` daemon requires access to `/dev/uinput` to inject key events
into your Wayland session. This file explains how to grant that access.

## Why This Is Needed

Vibe Attack creates a virtual keyboard device via the Linux `uinput` kernel module.
This allows key sequences to reach fullscreen Wayland games (like Helldivers 2) without
compositor-level shortcuts, which are blocked when a game holds exclusive focus.

## One-Time Setup

### 1. Load the uinput kernel module

```bash
sudo modprobe uinput
```

To persist across reboots:

```bash
echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf
```

### 2. Add yourself to the `input` group

> **Note for systemd v258+ users (Arch Linux / CachyOS 2025+):**
> Use the `input` group, **not** `uinput`. The `uinput` group was broken in systemd v258
> because non-system groups are no longer recognized by udev rules.

```bash
sudo usermod -aG input $USER
```

### 3. Apply the new group without logging out

```bash
newgrp input
```

Or log out and back in for a permanent session.

### 4. Verify access

```bash
ls -la /dev/uinput
# Should show group=input
groups | grep input && echo "OK: in input group" || echo "FAIL: not in input group"
```

## Troubleshooting

| Error | Likely Cause | Fix |
|-------|-------------|-----|
| permission denied on /dev/uinput | Not in input group | `sudo usermod -aG input $USER && newgrp input` |
| /dev/uinput not found | uinput module not loaded | `sudo modprobe uinput` |
| Still failing after group add | Session not refreshed | Log out and back in |
| CachyOS / systemd 258+ still failing | Wrong group (uinput not input) | Verify with `ls -la /dev/uinput` |

## Reference

- [Linux uinput documentation](https://www.kernel.org/doc/html/latest/input/uinput.html)
- [Arch Linux input group wiki](https://wiki.archlinux.org/title/Input_devices)
