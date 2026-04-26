#!/usr/bin/env bash
# vibe-attack setup script
# Automates first-time prerequisites for new users.
# Usage: setup.sh [--yes] [--dry-run] [--step=<step>] [--help]
set -euo pipefail

# ── Colour helpers ──────────────────────────────────────────────────────────
if [[ -t 1 ]]; then
  _bold='\033[1m'; _reset='\033[0m'; _green='\033[0;32m'; _red='\033[0;31m'; _yellow='\033[0;33m'
else
  _bold=''; _reset=''; _green=''; _red=''; _yellow=''
fi

ok()   { printf "${_green}  ✓${_reset} %s\n" "$*"; }
fail() { printf "${_red}  ✗${_reset} %s\n" "$*" >&2; }
info() { printf "${_bold}[%s]${_reset} %s\n" "$_current_step" "$*"; }
warn() { printf "${_yellow}  !${_reset} %s\n" "$*"; }

# ── Defaults ────────────────────────────────────────────────────────────────
YES=0
DRY_RUN=0
ONLY_STEP=""
_current_step="setup"

WHISPER_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin"

# ── XDG paths ───────────────────────────────────────────────────────────────
XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
CONFIG_TARGET="$XDG_CONFIG_HOME/vibe-attack/config.yaml"
MODEL_TARGET="$XDG_DATA_HOME/vibe-attack/models/whisper/ggml-tiny.en.bin"

# ── Script location (for finding config.example.yaml) ───────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_EXAMPLE="$REPO_ROOT/config.example.yaml"

# ── CLI parsing ─────────────────────────────────────────────────────────────
usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Set up vibe-attack prerequisites for a new user.

OPTIONS:
  -y, --yes        Non-interactive: answer yes to all prompts
      --dry-run    Print what would be done without executing
      --step=NAME  Run only the named step (for wizard integration)
      --help       Show this help and exit

STEPS (run in order):
  copy_config      Copy config.example.yaml to XDG config path
  install_model    Download whisper ggml-tiny.en model
  setup_uinput     Load uinput module and add user to input group
  validate         Verify all prerequisites are satisfied

EXAMPLES:
  $(basename "$0") --yes              # fully automated setup
  $(basename "$0") --dry-run          # preview actions without executing
  $(basename "$0") --step=setup_uinput  # run only uinput step
EOF
}

for arg in "$@"; do
  case "$arg" in
    -y|--yes)       YES=1 ;;
    --dry-run)      DRY_RUN=1 ;;
    --step=*)       ONLY_STEP="${arg#--step=}" ;;
    --help|-h)      usage; exit 0 ;;
    *) printf "Unknown option: %s\nRun with --help for usage.\n" "$arg" >&2; exit 1 ;;
  esac
done

# ── Utility ─────────────────────────────────────────────────────────────────
ask_confirm() {
  local prompt="$1"
  if [[ $YES -eq 1 ]]; then return 0; fi
  printf "%s [Y/n] " "$prompt"
  read -r reply
  [[ "${reply:-Y}" =~ ^[Yy]$ ]]
}

run_cmd() {
  if [[ $DRY_RUN -eq 1 ]]; then
    printf "  ${_yellow}dry-run:${_reset} %s\n" "$*"
    return 0
  fi
  "$@"
}

# ── Steps ───────────────────────────────────────────────────────────────────

step_copy_config() {
  _current_step="copy_config"
  info "Config file: $CONFIG_TARGET"

  if [[ -f "$CONFIG_TARGET" ]]; then
    ok "Config already exists — skipping"
    return 0
  fi

  if [[ ! -f "$CONFIG_EXAMPLE" ]]; then
    fail "config.example.yaml not found at $CONFIG_EXAMPLE"
    exit 1
  fi

  ask_confirm "Copy example config to $CONFIG_TARGET?" || { warn "Skipped by user"; return 0; }

  run_cmd mkdir -p "$(dirname "$CONFIG_TARGET")"
  run_cmd cp "$CONFIG_EXAMPLE" "$CONFIG_TARGET"
  ok "Config copied to $CONFIG_TARGET"
  info "Edit $CONFIG_TARGET to set your audio device and PTT key."
}

step_install_model() {
  _current_step="install_model"
  info "Whisper model: $MODEL_TARGET"

  if [[ -f "$MODEL_TARGET" ]] && [[ -s "$MODEL_TARGET" ]]; then
    ok "Model already installed — skipping"
    return 0
  fi

  if ! command -v curl &>/dev/null; then
    fail "curl not found. Download the model manually:"
    printf "    mkdir -p '%s'\n" "$(dirname "$MODEL_TARGET")"
    printf "    curl -L -o '%s' \\\\\n      '%s'\n" "$MODEL_TARGET" "$WHISPER_URL"
    exit 1
  fi

  info "URL: $WHISPER_URL"
  ask_confirm "Download whisper model (~75 MB) to $MODEL_TARGET?" || { warn "Skipped by user"; return 0; }

  run_cmd mkdir -p "$(dirname "$MODEL_TARGET")"
  run_cmd curl -L --progress-bar -o "$MODEL_TARGET" "$WHISPER_URL"
  ok "Model downloaded to $MODEL_TARGET"
}

step_setup_uinput() {
  _current_step="setup_uinput"
  local needs_relogin=0

  # Load uinput module
  if [[ -e /dev/uinput ]]; then
    ok "/dev/uinput exists — module already loaded"
  else
    info "Loading uinput kernel module"
    ask_confirm "Run: sudo modprobe uinput?" || { warn "Skipped modprobe — /dev/uinput will be missing"; }
    run_cmd sudo modprobe uinput

    # Persist across reboots
    if [[ ! -f /etc/modules-load.d/uinput.conf ]]; then
      ask_confirm "Persist uinput on boot (write /etc/modules-load.d/uinput.conf)?" && \
        run_cmd bash -c 'echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf >/dev/null'
    fi
    ok "uinput module loaded"
  fi

  # Add user to input group
  if getent group input | grep -q "\b${USER}\b" 2>/dev/null; then
    ok "User $USER already in input group"
  else
    info "Adding $USER to input group"
    warn "Note: use 'input' group (not 'uinput') — required for systemd v258+ / CachyOS 2025+"
    ask_confirm "Run: sudo usermod -aG input $USER?" || { warn "Skipped — /dev/uinput may not be accessible"; return 0; }
    run_cmd sudo usermod -aG input "$USER"
    needs_relogin=1
    ok "Added $USER to input group"
  fi

  if [[ $needs_relogin -eq 1 ]]; then
    warn "Log out and back in (or run 'newgrp input') for group membership to take effect."
  fi
}

step_validate() {
  _current_step="validate"
  local pass=0 fail_count=0

  check() {
    local label="$1" result="$2"
    if [[ "$result" == "ok" ]]; then
      ok "$label"
      ((pass++)) || true
    else
      fail "$label — $result"
      ((fail_count++)) || true
    fi
  }

  printf "\n${_bold}Validation${_reset}\n"

  # Config file
  if [[ -f "$CONFIG_TARGET" ]]; then
    check "Config file exists ($CONFIG_TARGET)" "ok"
  else
    check "Config file exists" "not found at $CONFIG_TARGET"
  fi

  # Model file
  if [[ -f "$MODEL_TARGET" ]] && [[ -s "$MODEL_TARGET" ]]; then
    check "Whisper model installed ($MODEL_TARGET)" "ok"
  else
    check "Whisper model installed" "not found or empty at $MODEL_TARGET"
  fi

  # /dev/uinput accessible
  if [[ -r /dev/uinput ]] && [[ -w /dev/uinput ]]; then
    check "/dev/uinput accessible (read+write)" "ok"
  elif [[ -e /dev/uinput ]]; then
    check "/dev/uinput accessible" "exists but not readable/writable — check group membership"
  else
    check "/dev/uinput accessible" "device not found — run setup_uinput step"
  fi

  # PTT key configured
  if [[ -f "$CONFIG_TARGET" ]] && grep -qE '^[[:space:]]*key:[[:space:]]+KEY_' "$CONFIG_TARGET"; then
    check "PTT key configured in config" "ok"
  else
    check "PTT key configured" "no 'key: KEY_*' found in $CONFIG_TARGET — edit ptt.key"
  fi

  printf "\n"
  if [[ $fail_count -eq 0 ]]; then
    printf "${_green}${_bold}SETUP COMPLETE${_reset} — vibe-attack is ready to run.\n"
    return 0
  else
    printf "${_red}${_bold}%d check(s) failed${_reset} — re-run the relevant steps above.\n" "$fail_count"
    return 1
  fi
}

# ── Step registry (ordered) ──────────────────────────────────────────────────
STEPS=(copy_config install_model setup_uinput validate)

# ── Main ────────────────────────────────────────────────────────────────────
main() {
  if [[ $DRY_RUN -eq 1 ]]; then
    warn "Dry-run mode — no changes will be made."
  fi

  if [[ -n "$ONLY_STEP" ]]; then
    case "$ONLY_STEP" in
      copy_config)   step_copy_config ;;
      install_model) step_install_model ;;
      setup_uinput)  step_setup_uinput ;;
      validate)      step_validate ;;
      *) fail "Unknown step: $ONLY_STEP"; printf "Valid steps: %s\n" "${STEPS[*]}" >&2; exit 1 ;;
    esac
    return
  fi

  for step in "${STEPS[@]}"; do
    printf "\n"
    "step_${step}"
  done
}

main
