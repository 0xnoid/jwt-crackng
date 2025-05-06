#!/usr/bin/env bash
set -e

# sudo?
run_with_sudo() {
    if [ "$EUID" -ne 0 ]; then
        sudo "$@"
    else
        "$@"
    fi
}

# Scope > Install Path
read -rp "Install jwt-crackng for current user only? [Y/n]: " choice
    if [[ "$choice" =~ ^[Nn] ]]; then
        INSTALL_SYSTEM=true
    else
        INSTALL_SYSTEM=false
    fi
    
    # Scope > The correct path, of course
    if $INSTALL_SYSTEM; then
        BIN_DIR="/usr/local/bin"
    else
        BIN_DIR="$HOME/.local/bin"
        mkdir -p "$BIN_DIR"
    fi

# Download executable release
URL="https://github.com/0xnoid/jwt-crackng/releases/download/v0.2.0/jwt-crackng"
TMPFILE="$(mktemp)"
    echo "Downloading jwt-crackng from $URL…"
    curl -fsSL -o "$TMPFILE" "$URL"

chmod +x "$TMPFILE"
    if $INSTALL_SYSTEM; then
        echo "Installing system-wide to $BIN_DIR…"
        run_with_sudo mv "$TMPFILE" "$BIN_DIR/jwt-crackng"
    else
        echo "Installing for current user to $BIN_DIR…"
        mv "$TMPFILE" "$BIN_DIR/jwt-crackng"
    fi

echo
echo "✔ jwt-crackng installed to $BIN_DIR/jwt-crackng"
echo

# Scope > Add to $PATH
echo "Add $BIN_DIR to your PATH?"
echo "  1) Bash"
echo "  2) Zsh"
echo "  3) Fish"
echo "  S) Skip"
read -rp "Select [1/2/3/S]: " shell_choice

case "$shell_choice" in
    1)
        if $INSTALL_SYSTEM; then
            rc_file="/etc/bash.bashrc"
        else
            rc_file="$HOME/.bashrc"
        fi
        ;;
    2)
        if $INSTALL_SYSTEM; then
            rc_file="/etc/zshrc"
        else
            rc_file="$HOME/.zshrc"
        fi
        ;;
    3)
        if $INSTALL_SYSTEM; then
            rc_file="/etc/fish/config.fish"
        else
            rc_file="$HOME/.config/fish/config.fish"
        fi
        ;;
    [Ss])
        echo "Skipping PATH update."
        exit 0
        ;;
    *)
        echo "Invalid choice; skipping."
        exit 0
        ;;
esac

# Scope > $DIR
if $INSTALL_SYSTEM; then
    run_with_sudo mkdir -p "$(dirname "$rc_file")"
else
    mkdir -p "$(dirname "$rc_file")"
fi

# Scope > Append to $PATH
export_line="export PATH=\"$BIN_DIR:\$PATH\""
if $INSTALL_SYSTEM; then
    run_with_sudo bash -c "echo '$export_line' >> '$rc_file'"
else
    echo "$export_line" >> "$rc_file"
fi

echo
echo "✔ Added:"
echo "  $export_line"
echo "to $rc_file"
echo
echo "Restart or run 'source $rc_file' to pick up your updated PATH."
