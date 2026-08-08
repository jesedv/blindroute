#!/bin/sh
set -e

# BlindRoute installer — one-line FHE CLI install
# Usage: curl -fsSL https://raw.githubusercontent.com/jesedv/blindroute/master/install.sh | sh

REPO="jesedv/blindroute"
BIN="blindroute"
VERSION="${VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

main() {
    detect_platform
    determine_url
    install_binary
    verify
    print_success
}

detect_platform() {
    OS=$(uname -s)
    ARCH=$(uname -m)

    case "$OS" in
        Linux)  PLATFORM="x86_64-linux" ;;
        Darwin) PLATFORM="x86_64-linux" ;;  # macOS uses same binary via Rosetta or native via cargo
        *)
            echo "Unsupported OS: $OS"
            echo "Install from source: cargo install blindroute"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64) ARCH_DIR="x86_64" ;;
        aarch64|arm64) ARCH_DIR="aarch64" ;;
        *)
            echo "Unsupported architecture: $ARCH"
            echo "Install from source: cargo install blindroute"
            exit 1
            ;;
    esac
}

determine_url() {
    if [ "$VERSION" = "latest" ]; then
        API="https://api.github.com/repos/$REPO/releases/latest"
        if command -v curl >/dev/null 2>&1; then
            URL=$(curl -fsSL "$API" | grep "browser_download_url.*$PLATFORM" | head -1 | cut -d '"' -f 4)
        elif command -v wget >/dev/null 2>&1; then
            URL=$(wget -qO- "$API" | grep "browser_download_url.*$PLATFORM" | head -1 | cut -d '"' -f 4)
        fi
    else
        URL="https://github.com/$REPO/releases/download/$VERSION/$BIN-$PLATFORM.tar.gz"
    fi

    if [ -z "$URL" ]; then
        echo "Could not find release for platform: $PLATFORM"
        echo "Install from source: cargo install blindroute"
        exit 1
    fi

    echo "Downloading $BIN $VERSION for $PLATFORM..."
}

install_binary() {
    mkdir -p "$INSTALL_DIR"
    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$URL" -o "$TMPDIR/$BIN.tar.gz"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$URL" -O "$TMPDIR/$BIN.tar.gz"
    else
        echo "Need curl or wget to download."
        exit 1
    fi

    tar -xzf "$TMPDIR/$BIN.tar.gz" -C "$TMPDIR"
    install -m 755 "$TMPDIR/$BIN" "$INSTALL_DIR/$BIN"
    echo "Installed to $INSTALL_DIR/$BIN"
}

verify() {
    if ! "$INSTALL_DIR/$BIN" >/dev/null 2>&1; then
        echo "Binary verification failed."
        exit 1
    fi
}

print_success() {
    echo ""
    echo "✓ BlindRoute installed successfully!"
    echo ""
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo "Add $INSTALL_DIR to your PATH:"
        echo "  export PATH=\"\$INSTALL_DIR:\$PATH\""
        echo ""
    fi
    echo "Quick start:"
    echo "  blindroute                     # self-test"
    echo "  blindroute keygen --out keys/  # generate keys"
    echo "  blindroute help                # all commands"
    echo ""
    echo "→ https://blindroute.pages.dev"
}

main "$@"
