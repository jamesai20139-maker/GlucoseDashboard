#!/usr/bin/env bash

set -e

echo "=========================================="
echo " GlucoseDashboard WSL Environment Setup"
echo "=========================================="
echo

# ------------------------------------------
# 1. 檢查 Ubuntu / WSL
# ------------------------------------------

if ! command -v apt >/dev/null 2>&1; then
    echo "ERROR: This script requires an Ubuntu/Debian environment."
    exit 1
fi

echo "[1/5] Updating apt package list..."
sudo apt update

# ------------------------------------------
# 2. 安裝基本開發工具
# ------------------------------------------

echo
echo "[2/5] Installing basic development tools..."

sudo apt install -y \
    build-essential \
    make \
    curl \
    git \
    pkg-config \
    libssl-dev

# ------------------------------------------
# 3. 安裝 Node.js / npm
# ------------------------------------------

echo
echo "[3/5] Checking Node.js / npm..."

if command -v node >/dev/null 2>&1 && command -v npm >/dev/null 2>&1; then

    echo "Node.js already installed:"
    node --version

    echo "npm already installed:"
    npm --version

else

    echo "Node.js / npm not found."
    echo "Installing Node.js from Ubuntu packages..."

    sudo apt install -y nodejs npm

fi

# ------------------------------------------
# 4. 安裝 Rust / Cargo
# ------------------------------------------

echo
echo "[4/5] Checking Rust / Cargo..."

if command -v cargo >/dev/null 2>&1; then

    echo "Cargo already installed:"
    cargo --version

else

    echo "Cargo not found."
    echo "Installing Rust using rustup..."

    curl --proto '=https' \
         --tlsv1.2 \
         -sSf https://sh.rustup.rs \
         | sh -s -- -y

    # 載入 Rust 環境
    source "$HOME/.cargo/env"

fi

# ------------------------------------------
# 5. 最終檢查
# ------------------------------------------

echo
echo "[5/5] Checking installed tools..."
echo

check_command() {

    if command -v "$1" >/dev/null 2>&1; then
        printf "  %-10s OK\n" "$1"
    else
        printf "  %-10s MISSING\n" "$1"
    fi

}

check_command git
check_command make
check_command curl
check_command node
check_command npm
check_command rustc
check_command cargo

echo
echo "=========================================="
echo " Version Information"
echo "=========================================="

echo
echo "Git:"
git --version

echo
echo "Make:"
make --version | head -n 1

echo
echo "Node:"
node --version

echo
echo "npm:"
npm --version

echo
echo "Rust:"
rustc --version

echo
echo "Cargo:"
cargo --version

echo
echo "=========================================="
echo " Setup completed!"
echo "=========================================="
echo
echo "You can now run:"
echo
echo "    make run"
echo