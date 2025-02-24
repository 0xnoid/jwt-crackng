#!/bin/bash
set -e

echo "Building jwt-crackng..."
cargo build --release

INSTALL_DIR="/usr/local/bin"
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "Adding $INSTALL_DIR to PATH..."
    echo "export PATH=\$PATH:$INSTALL_DIR" >> ~/.bashrc
    source ~/.bashrc
fi

echo "Installing jwt-crackng..."
sudo cp target/release/jwt-crackng "$INSTALL_DIR"

echo "Installation complete! You can now use 'jwt-crackng' globally."
