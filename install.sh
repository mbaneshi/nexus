#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${HOME}/.local/bin"

echo "Building nexus (release mode)..."
cargo build --release -p nexus-cli

echo "Installing nexus to ${INSTALL_DIR}..."
mkdir -p "${INSTALL_DIR}"
cp target/release/nexus "${INSTALL_DIR}/nexus"

echo "Done. Make sure ${INSTALL_DIR} is in your PATH."
echo ""
echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
echo ""
echo "Run 'nexus --help' to get started."
