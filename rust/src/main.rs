//! `cargo run --release` — the Rust replacement for `python main.py` (phase 8).
//!
//! Runs [`PANELS`] in `main.py`'s order and writes each panel's text to stdout as soon as it is
//! done, so a long run shows its progress the way the Python did. The text is held byte-equal to
//! `python main.py`'s PyPy stdout by `tests/cli_golden.rs`.

use std::io::Write;

use turbojet::panels::{Design, PANELS};
use turbojet::pyfmt::Printer;

fn main() {
    let design = Design::new();
    let mut printer = Printer::new();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for (_, panel) in PANELS {
        panel(&mut printer, &design);
        out.write_all(printer.take().as_bytes()).expect("stdout closed");
        out.flush().expect("stdout closed");
    }
}
