# Installation of Rust and tools for Linux

Make sure your system has `curl`, `gcc`, `git` and common build tools installed, e.g., On debian based system:

```bash
sudo apt update
sudo apt install git curl build-essential pkg-config ca-certificates -y
```

Use the official installer `rustup`, or any alternative method on https://www.rust-lang.org/tools/install (by your own):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

- Follow the prompts in the terminal.
- After installation, if not shown as done run:

```bash
echo "source '$HOME/.cargo/env'" >> ~/.bashrc
source ~/.bashrc
```

## Test Your Setup

- Verify the rust installation:

```bash
rustc --version
```

Run:

```bash
echo "Rust is awesome! 🦀🔥🚀"
```

You should see emojis rendered correctly in your terminal or text editors. If not follow the instructions below to
enable emoji support.

- Then follow [run instructions](README.md#running)

---

## Enable Emoji Font Support

To properly display emoji characters in your terminal and system fonts, install an emoji-compatible font.

### For Ubuntu/Debian-based distros:

```bash
sudo apt install fonts-noto-color-emoji
```

### For Arch Linux:

```bash
sudo pacman -S noto-fonts-emoji
```

### For Fedora:

```bash
sudo dnf install google-noto-emoji-color-fonts
```

---

### Font Fallback if emojis still do not render

Create or edit the following file:

```bash
~/.config/fontconfig/fonts.conf
```

Add:

```xml
<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
    <alias>
        <family>sans-serif</family>
        <prefer>
            <family>Noto Color Emoji</family>
        </prefer>
    </alias>
</fontconfig>
```

Then refresh the font cache:

```bash
fc-cache -f -v
```

---
