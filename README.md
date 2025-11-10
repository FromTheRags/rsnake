[![crates.io](https://img.shields.io/crates/v/rsnaker.svg)](https://crates.io/crates/rsnaker)
[![Doc.rs](https://docs.rs/rsnaker/badge.svg)](https://docs.rs/rsnaker)
[![GPLv3 License](https://img.shields.io/badge/License-GPL%20v3-yellow.svg)](https://opensource.org/licenses/)
[![Last Commit](https://img.shields.io/github/last-commit/fromtherags/rsnake)](https://github.com/FromTheRags/rsnake/commits)
[![Build](https://github.com/FromTheRags/rsnake/actions/workflows/build.yml/badge.svg)](https://github.com/FromTheRags/rsnake/actions/workflows/build.yml)
[![Test](https://github.com/FromTheRags/rsnake/actions/workflows/test.yml/badge.svg)](https://github.com/FromTheRags/rsnake/actions/workflows/test.yml)
[![Last doc](https://img.shields.io/badge/docs-online-blue)](https://fromtherags.github.io/rsnake/rsnaker/index.html)
[![Code lines](https://img.shields.io/badge/lines%20of%20code-3K-blue)](https://github.com/FromTheRags/rsnake)
<!-- 
[![Code lines](https://tokei.rs/b1/github/FromTheRags/rsnake)](https://github.com/FromTheRags/rsnake)
As tokei server seems unreliable with messages as "Invalid SHA provided."/bandwidth issues and not actively maintained 
, the best is yet a static badge yielding the total amount of code get from the command:`tokei`
in a local terminal (having a GitHub action with writing privilege just for that is overkill) -->

# Snake Game using Ratatui

It is a terminal-based snake game using the Ratatui crate for rendering.
![Playing Snake](demo_images/demo.gif)
![Terminal Welcome Menu](demo_images/snake_welcome.png)
![Terminal Output Menu](demo_images/snake_setup.png)
![Terminal Output Running](demo_images/snake_running.png)

## Features

- **Terminal UI**: Uses Ratatui for rendering a grid-based game.
- **Game Logic**: Manages snake movement, collisions, and scoring.
- **Multithreading**: Uses multiple threads for input handling, rendering at 60 FPS, and game logic execution.
- **Emoji-based graphics**: Supports rendering the snake using emojis instead of ASCII.
- **Configurable parameters**: With `clap` for command-line arguments & toml file.

## TODO

- [ ] Add a save score (local db) with a pseudo got from cmdline
- [ ] Add preset for saved configuration
- [ ] Add timers to fruits ⏲️
- [ ] Add some performance logs with tracing,
  for [example](https://github.com/ratatui/ratatui/blob/main/examples/apps/tracing/src/main.rs)
- [ ] Enhance all table parameters based on the generic one
- [ ] Internal code: Provide a macro example for the trait implementation widget and this error use.

## 💖 Support

- 🥤If you like this game and want to contribute to more amazing features, consider giving me a coffee to support
  development:
  [![Support me on Ko-fi](https://img.shields.io/badge/Ko--fi-Support%20Me-ff5f5f?logo=kofi&logoColor=white)](https://ko-fi.com/retrosnake)

## Run Game options

- To see run options, use: `rsnake --help`
- E.g., `rsnake -z 🐼 -b 🍥` or `cargo run -- -z 🐼 -b 🍥` (if from source)
- To save a set of commands, create and alias or use `--save` to generate to a snake_config.toml file near the
  executable.
- Then load with `--load` option. [Shipped one](snake_config.toml)

## Installation from release

### ✅ Prerequisites

- 💻 **Use a terminal that supports emoji**
    - On **Windows**,the [new microsoft terminal](https://apps.microsoft.com/detail/9n0dx20hk701?hl=en-us&gl=US) shipped
      with w11 (and compatible w10) supports emoji out of the box among other improvements.
    - On **Linux** / Android, you could need to install the Noto Emoji font:
      👉 [Emoji font support](#Enable-Emoji-Font-Support) for instructions.
    - Some screen displays can flicker at 60 FPS in the terminal, use a decent display or an external monitor.

### Running

- Download the latest release from the [release page](https://github.com/FromTheRags/rsnake/releases) according to your
  OS.
- Run the executable using the terminal or double-click on the file if your OS supports it.
- For windows:
    - Search for "Terminal" in the search menu to launch it and **set as default** (to be able to run the snake by
      double-clicking the .exe) or run rsnake from the terminal
      with:
    - `cd "download path"` then `.\rsnake-x86_64-pc-windows-msvc.exe`
- For Linux/macOS/android:
    - `cd "download path" && ./rsnake-x86_64-unknown-linux-gnu` (or `./rsnake-x86_64-apple-darwin` on macOS)
    - Or use the executable directly if you have it in your PATH.
- See [Run option below for more details](#Run-Game-options).

## Installation from source

### ✅ Prerequisites

- 🦀 **Have Rust and compilation tools installed**  
  If you don't have Rust yet:
    - On **Windows**, Install Rust using the official .exe installer https://www.rust-lang.org/tools/install (as it
      works
      Out-Of-The-Box on windows)
    - On **Linux** 👉 [Installation Rust and tools for Linux](#Installation-of-Rust-and-tools-for-Linux) for instructions
    - On **Android** use a linux emulator and follow the same instruction as linux, tested with:
        - [Android using Ubuntu](https://github.com/CypherpunkArmory/UserLAnd),
        - [Termux](https://github.com/termux/termux-app)  
          For easier use, END key works as ENTER and HOME as a pause key.

### Running

- Clone this repository
  `git clone https://github.com/FromTheRags/rsnake.git`
- Go to the directory `cd rsnake`
- To run the game, either:`cargo run` or `cargo run --manifest-path rsnake/Cargo.toml` (if in another directory)
- To install the game as a command:  
  `cargo install --path .`  
  And then run the game with: `rsnake`
- See [Run option below for more details](#Run-Game-options).

## Architecture

- Uses `Arc` & `RwLock` for synchronization.
- Spawns separate threads for input handling, rendering (60 Hz), and game logic execution.

## Documentation generation

- `cargo doc --document-private-items --no-deps --open`

## Tests

- As usual run them with `cargo test` the project is set up with a lib containing all the code, and a main.rs just
  calling it
- As this is a widespread pattern providing full compliance with the Rust test ecosystem, allowing doc comment to be
  automatically tested, for example.
- To have a coverage report, install `llvm-cov`:
  `rustup component add llvm-tools-preview
  cargo install cargo-llvm-cov`
- And run `cargo llvm-cov --open`
- A great coverage is not a goal for this project (tests are only there to showcase tests in rust),
- For reference, the current coverage is :
  [![codecov](https://codecov.io/gh/fromtherags/rsnake/branch/main/graph/badge.svg)](https://codecov.io/gh/FromTheRags/rsnake)

## Installation of Rust and tools for Linux

Make sure your system has `curl`, `gcc` and `git` installed:

```bash
sudo apt update
sudo apt install curl git gcc -y
```

Use the official installer `rustup`, or any alternative method on https://www.rust-lang.org/tools/install (by your own):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

- Follow the prompts in the terminal.
- After installation, run:

```bash
echo "source '$HOME/.cargo/env'" >> ~/.bashrc
source ~/.bashrc
```

- Verify the installation:

```bash
rustc --version
```

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

## Optional: Configure Font Fallback (if emojis still do not render)

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

## Test Your Setup

Run:

```bash
echo "Rust is awesome! 🦀🔥🚀"
```

You should see emojis rendered correctly in your terminal or text editors.

- Then follow quick installation instructions

## References

- Clippy lints: <https://github.com/rust-lang/rust-clippy/>
- Ratatui tutorial: <https://ratatui.rs/tutorials/hello-world/>
- Example: <https://ratatui.rs/examples/widgets/canvas/>
- Git over-bloated: <https://rtyley.github.io/bfg-repo-cleaner/>
- Gif: [OBS studio](https://obsproject.com/fr/download) full screen with filter to terminal view and
  [ffmpeg](https://ffmpeg.org/)
  with:  
  `ffmpeg -ss 00:00:01 -to 00:00:20 -i input.mp4 -vf "fps=10,scale=1400:-1:flags=lanczos" -loop 0 output.gif`  
  (well better than asciinema)  
  [![Contributors](https://img.shields.io/github/contributors/fromtherags/rsnake)](https://github.com/fromtherags/rsnake/graphs/contributors)
