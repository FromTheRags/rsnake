# Comprehensive Guide: Testing macOS (Mach-O) Programs on Windows using WSL 2 & Darling
### Practical Case: Emulating `rsnake` (FromTheRags) Terminal Game

This guide provides an end-to-end tutorial for running and testing macOS command-line applications (Mach-O 64-bit `x86_64-apple-darwin` binaries such as **rsnake**) on a Windows machine without owning a physical Mac, using **WSL 2** and the **Darling** translation layer.

---

## Table of Contents
1. [Overall Architecture](#1-overall-architecture)
2. [WSL 2 & Ubuntu Setup](#2-wsl-2--ubuntu-setup)
3. [Darling Installation (macOS Compatibility Layer)](#3-darling-installation-macos-compatibility-layer)
4. [Eliminating Unwanted Warnings (Purging X11 / dyld Errors)](#4-eliminating-unwanted-warnings-purging-x11--dyld-errors)
5. [Obtaining the macOS Binary for rsnake](#5-obtaining-the-macos-binary-for-rsnake)
   - [Option A: Downloading the Pre-built Binary (Recommended)](#option-a-downloading-the-pre-built-binary-recommended)
   - [Option B: Cross-compiling from Source (Rust + macOS SDK)](#option-b-cross-compiling-from-source-rust--macos-sdk)
6. [Running & Testing the Application with Darling](#6-running--testing-the-application-with-darling)
7. [Troubleshooting & FAQ](#7-troubleshooting--faq)
8. [Official Links & References](#8-official-links--references)

---

## 1. Overall Architecture

```text
[ Windows 10/11 ]
       │
       ▼
[ WSL 2 (Native Linux Kernel) ]
       │
       ▼
[ Ubuntu 26.04 / 24.04 LTS ]
       │
       ▼
[ Darling (darlingserver + overlayfs + macOS dyld) ]
       │
       ▼
[ Mach-O 64-bit Binary: rsnake (FromTheRags) ]
```

- **WSL 2**: Provides a lightweight utility virtual machine powered by an authentic, highly optimized Microsoft Linux kernel.
- **Darling**: An open-source translation layer (analogous to Wine on Linux for Windows software). It provides clean userspace implementations of Apple frameworks (CoreFoundation, Cocoa, libSystem) and the Darwin Mach-O dynamic linker `dyld`.
- **rsnake**: A retro arcade terminal Snake game written in Rust using the *Ratatui* TUI crate, developed by *FromTheRags*.

---

## 2. WSL 2 & Ubuntu Setup

### 2.1. Enable WSL 2
Open Windows PowerShell as **Administrator**:

```powershell
# Set WSL 2 as the default architecture
wsl --set-default-version 2
```

If WSL was not previously installed on your system:
```powershell
wsl --install --no-launch
```
*(A system reboot might be required if Hyper-V or Virtual Machine Platform features were freshly enabled).*

### 2.2. List and Install Ubuntu
Check available online distributions:
```powershell
wsl --list --online
```

Install Ubuntu 26.04 LTS (or Ubuntu 24.04 LTS):
```powershell
wsl --install Ubuntu-26.04 --no-launch
```

### 2.3. Verify Distribution Status
```powershell
wsl -l -v
```
You should see version `2` confirmed:
```text
  NAME            STATE           VERSION
* Ubuntu-26.04    Running         2
```

To enter your Ubuntu instance:
```powershell
wsl -d Ubuntu-26.04
```

---

## 3. Darling Installation (macOS Compatibility Layer)

Darling operates entirely in userspace using `darlingserver` and FUSE/overlayfs.

### 3.1. System Update & Dependencies
Inside your WSL terminal (`wsl -d Ubuntu-26.04`):

```bash
sudo apt-get update && sudo apt-get upgrade -y

# Install essential build tools, FUSE, 32-bit compatibility runtime, and compilers
sudo apt-get install -y \
  curl wget unzip file build-essential git \
  libfuse2t64 fuse3 libc6-i386 \
  pkg-config cmake clang lld python3
```

### 3.2. Download Official Darling Debian Packages
Fetch the official `.deb` bundle from the Darling project releases:

```bash
mkdir -p /tmp/darling-install && cd /tmp/darling-install

# Download official release bundle
wget https://github.com/darlinghq/darling/releases/download/v0.1.20260608/debs_20260608.zip
unzip debs_20260608.zip
cd debs_20260609
```

### 3.3. Install CLI & Core Packages
For running command-line / TUI applications, install only the core and CLI packages:

```bash
sudo dpkg -i \
  darling-core_*.deb \
  darling-system_*.deb \
  darling-ffi_*.deb \
  darling-cli-gui-common_*.deb \
  darling-cli-python2-common_*.deb \
  darling-cli_*.deb \
  darling-cli-extra_*.deb \
  darling-jsc-webkit-common_*.deb \
  darling-jsc_*.deb
```

---

## 4. Eliminating Unwanted Warnings (Purging X11 / dyld Errors)

### 4.1. Why Did These Errors Occur?
If `darling-iokitd` or GUI modules are installed, Darling's `launchd` service tries to load an X11 / OpenGL graphics daemon (`libX11.dylib`, `libXRandR.dylib`) every time the container starts.

In a terminal environment without a native X11 wrapper, this results in polluting logs:
```text
dyld: Library not loaded: /usr/lib/native/libX11.dylib
  Referenced from: /usr/sbin/iokitd
  Reason: image not found
abort_with_payload: reason: dyld: No shared cache present
```

### 4.2. How to Ensure a Clean 100% Silent Environment
For CLI and TUI applications like `rsnake`, this daemon is completely unnecessary.

Purge the graphical packages and shut down running container instances:
```bash
# 1. Purge the IOKit graphics daemon packages
sudo dpkg --purge darling-iokitd darling-iosurface

# 2. Stop any running background container instances
darling shutdown
```

### 4.3. Verification
```bash
darling shell echo "Darling is clean and operational"
```
Expected output:
```text
Darling is clean and operational
```
*(No warnings, no dyld errors).*

---

## 5. Obtaining the macOS Binary for rsnake

Darling runs **Mach-O 64-bit (`x86_64-apple-darwin`)** binaries. You have two options to obtain this binary.

### Option A: Downloading the Pre-built Binary (Recommended)
The author of `rsnake` (*FromTheRags*) provides pre-compiled binaries in the GitHub Releases.

Download the latest release (**v0.4.2**):
```bash
sudo curl -L -o /usr/local/bin/rsnake-darwin \
  https://github.com/FromTheRags/rsnake/releases/download/v0.4.2/rsnake-x86_64-apple-darwin

sudo chmod +x /usr/local/bin/rsnake-darwin
```

Verify the binary signature:
```bash
file /usr/local/bin/rsnake-darwin
```
Expected output:
```text
/usr/local/bin/rsnake-darwin: Mach-O 64-bit x86_64 executable, flags:<NOUNDEFS|DYLDLINK|TWOLEVEL|PIE|HAS_TLV_DESCRIPTORS>
```

---

### Option B: Cross-compiling from Source (Rust + macOS SDK)

If you are developing your own Rust application or want to build `rsnake` yourself from source:

#### 1. Install the Rust Toolchain & Target
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Add the 64-bit macOS Darwin target
rustup target add x86_64-apple-darwin
```

#### 2. Download the Official macOS SDK
The linker requires Apple system symbols (`CoreFoundation`, `libSystem`, `libiconv`) provided in the official SDK:
```bash
sudo mkdir -p /opt/sdk && cd /opt/sdk
sudo wget https://github.com/joseluisq/macosx-sdks/releases/download/11.3/MacOSX11.3.sdk.tar.xz
sudo tar -xf MacOSX11.3.sdk.tar.xz
```

#### 3. Configure Clang and LLD as the Linker
Create a wrapper script at `/usr/local/bin/x86_64-apple-darwin-clang`:
```bash
sudo bash -c 'cat << "EOF" > /usr/local/bin/x86_64-apple-darwin-clang
#!/bin/bash
exec clang -fuse-ld=lld -target x86_64-apple-darwin -isysroot /opt/sdk/MacOSX11.3.sdk -L/opt/sdk/MacOSX11.3.sdk/usr/lib -F/opt/sdk/MacOSX11.3.sdk/System/Library/Frameworks "$@"
EOF'

sudo chmod +x /usr/local/bin/x86_64-apple-darwin-clang
sudo ln -sf /usr/bin/llvm-ar-21 /usr/local/bin/llvm-ar 2>/dev/null || sudo ln -sf $(which llvm-ar) /usr/local/bin/llvm-ar
```

Configure Cargo (`~/.cargo/config.toml`):
```toml
[target.x86_64-apple-darwin]
linker = "/usr/local/bin/x86_64-apple-darwin-clang"
ar = "llvm-ar"
```

#### 4. Clone and Build rsnake
```bash
git clone https://github.com/FromTheRags/rsnake.git
cd rsnake
cargo build --target x86_64-apple-darwin --release

# The compiled Mach-O binary will be at:
# target/x86_64-apple-darwin/release/rsnake
```

---

## 6. Running & Testing the Application with Darling

### 6.1. Direct Execution from Windows PowerShell
You can execute the binary directly from Windows without manually entering the Linux shell:

```powershell
# Check version
wsl -d Ubuntu-26.04 darling /usr/local/bin/rsnake-darwin --version

# View help and options
wsl -d Ubuntu-26.04 darling /usr/local/bin/rsnake-darwin --help
```

### 6.2. Play with Custom Options
```powershell
# Custom snake symbols and speed
wsl -d Ubuntu-26.04 darling /usr/local/bin/rsnake-darwin -s fast -z 🐼 -b 🍥
```

In-game Keybindings:
- **Arrow Keys**: Move the snake.
- **- (Hyphen)**: Pause / Unpause.
- **HOME**: Open the in-game settings menu.
- **TAB**: Quit the game.

### 6.3. Create a Seamless Wrapper Script (`rsnake`)
In Ubuntu WSL, create `/usr/local/bin/rsnake`:
```bash
sudo bash -c 'cat << "EOF" > /usr/local/bin/rsnake
#!/bin/bash
exec darling /usr/local/bin/rsnake-darwin "$@"
EOF'

sudo chmod +x /usr/local/bin/rsnake
```

Now you can run it cleanly from anywhere in Windows:
```powershell
wsl -d Ubuntu-26.04 rsnake
```

### 6.4. Explore the Interactive Darling macOS Shell
To inspect the virtual macOS environment:
```powershell
wsl -d Ubuntu-26.04 darling shell
```
Inside the Darling shell:
```bash
# Check emulated macOS version
sw_vers
# Output:
#   ProductName:    macOS
#   ProductVersion: 11.7.4 (Big Sur)
#   BuildVersion:   Darling

# Test native Darwin commands
uname -a
whoami

# Exit back to Linux
exit
```

---

## 7. Troubleshooting & FAQ

### Q1: "Bootstrapping the container with launchd..." appears
- **Cause**: Darling initializes its chroot / overlayfs mount namespace on the first invocation after system boot or after running `darling shutdown`.
- **Normal Behavior**: This message only appears once when the background daemon starts. All subsequent commands run immediately without printing this message.

### Q2: When should I run `darling shutdown`?
- Run `darling shutdown` whenever you modify system files under `/usr/libexec/darling`, or if a background Darwin process becomes unresponsive. The container will automatically reinitialize cleanly on the next command.

### Q3: Terminal emoji display issues
- Ensure your terminal uses an emoji/symbol-compatible monospace font (e.g., **Cascadia Code**, **FiraCode Nerd Font**, or **Noto Color Emoji**) in your Windows Terminal profile settings.

---

## 8. Official Links & References

- **Darling Official Documentation**: [https://docs.darlinghq.org](https://docs.darlinghq.org)
- **Darling GitHub Repository**: [https://github.com/darlinghq/darling](https://github.com/darlinghq/darling)
- **rsnake GitHub Repository (FromTheRags)**: [https://github.com/FromTheRags/rsnake](https://github.com/FromTheRags/rsnake)
- **rsnake GitHub Releases**: [https://github.com/FromTheRags/rsnake/releases](https://github.com/FromTheRags/rsnake/releases)
- **Microsoft WSL Official Documentation**: [https://learn.microsoft.com/windows/wsl/](https://learn.microsoft.com/windows/wsl/)
- **Apple macOS SDK Archives**: [https://github.com/joseluisq/macosx-sdks](https://github.com/joseluisq/macosx-sdks)
