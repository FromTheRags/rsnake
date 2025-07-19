#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use rsnaker::start_snake;
// Project size:
// Basic to get the number of lines without comments and blanks:
// find . -name '*.rs' -not -path "./target/*" -print0 | xargs -0 cat | grep -v '^\s*$' | grep -v '^\s*//' | wc -l
// including comments without blanks:
// find . -name '*.rs' -not -path "./target/*" -print0 | xargs -0 cat | grep -v '^\s*$' | wc -l
// To get all in lines in .rs:
// find . -name '*.rs' -print0 | xargs -0 cat | wc -l
// for more precise analyse: cargo install tokei && tokei
// To have an overview of ratatui capabilities:
// https://ratatui.rs/concepts/widgets/
// https://github.com/ratatui/awesome-ratatui
// https://ratatui.rs/showcase/third-party-widgets/

fn main() {
    start_snake();
}
