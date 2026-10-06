//! `cargo run --release` — the Rust replacement for `python main.py` (phase 8).
//!
//! Runs [`PANELS`] in `main.py`'s order and writes each panel's text to stdout as soon as it is
//! done, so a long run shows its progress the way the Python did. The text is held byte-equal to
//! `python main.py`'s PyPy stdout by `tests/cli_golden.rs` — except its last line, which names
//! the chart DATA this binary writes (`ts_diagram.json`, in the working directory) rather than a
//! PNG it does not draw: `python plot_ts_diagram.py` draws it (slice AR).
//!
//! Two subcommands replace `docs/visuals/`'s scripts, and a third writes the chart data alone
//! (slice AR):
//!
//! * `visuals [DIR]` — `extract_data.py` + `build.py` + `build_cutaway.py`: run the model, write
//!   `data.json`, and splice both pages. `DIR` defaults to the repo's `docs/visuals`; the
//!   templates are always read from there.
//! * `splice [DIR]` — `build.py` + `build_cutaway.py` alone: re-splice both pages from the
//!   committed `data.json` (a template-only edit needs no model run).
//! * `ts-diagram` — write `ts_diagram.json` alone, without the ~10-minute panel run.
//! * `panel NAME [--write]` — run the panel(s) named `NAME` alone and print them. With `--write`
//!   the one panel must be [`RUST_OWNED`], and its text is written to
//!   `rust/oracle/rust_owned/NAME.txt` — the ONLY way that capture is made (the file is written
//!   here, byte for byte, so no shell redirect can re-encode it).
//! * `sandbox WASM [OUT]` — splice the browser build `WASM` into `docs/sandbox/template.html`,
//!   writing `OUT` (default `docs/sandbox/turbojet-sandbox.html`). `rust/sandbox-wasm/build.ps1`
//!   runs it; see `docs/sandbox/README.md`.
//! * `sandbox-native` — print each of the sandbox check grid's requests and its native reply,
//!   `REQUEST<TAB>REPLY` per line, for the browser check to compare against
//!   (`rust/sandbox-wasm/check.mjs`).

use std::io::Write;
use std::path::{Path, PathBuf};

use turbojet::panels::{Design, PANELS, RUST_OWNED, TS_DIAGRAM_JSON};
use turbojet::pyfmt::Printer;
use turbojet::sandbox;
use turbojet::visuals;

fn visuals_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("docs").join("visuals")
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

fn write(p: &Path, text: &str) {
    std::fs::write(p, text).unwrap_or_else(|e| panic!("cannot write {}: {e}", p.display()));
}

/// Splice both pages from `data_text` (exactly the bytes of `data.json`) into `out`.
fn splice_pages(data_text: &str, out: &Path) {
    let src = visuals_dir();
    let data = visuals::Json::parse(data_text);
    let page = visuals::splice_visuals(&read(&src.join("template.html")), data_text);
    write(&out.join("turbojet-visuals.html"), &page);
    println!("built turbojet-visuals.html ({} bytes)", page.len());
    let cut = visuals::splice_cutaway(&read(&src.join("cutaway-template.html")), &data);
    write(&out.join("turbojet-cutaway.html"), &cut);
    println!("{} bytes -> {}", cut.len(), out.join("turbojet-cutaway.html").display());
}

fn write_ts_diagram(design: &Design) {
    let ts = visuals::ts_diagram_json(&visuals::ts_diagram(design)).dump();
    write(Path::new(TS_DIAGRAM_JSON), &ts);
}

fn run_panels() {
    let design = Design::new();
    // The chart data first: it needs only the design point, and a long run interrupted later
    // still leaves it behind.
    write_ts_diagram(&design);
    let mut printer = Printer::new();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for (_, panel) in PANELS {
        panel(&mut printer, &design);
        out.write_all(printer.take().as_bytes()).expect("stdout closed");
        out.flush().expect("stdout closed");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out_dir = |i: usize| args.get(i).map(PathBuf::from).unwrap_or_else(visuals_dir);
    match args.first().map(String::as_str) {
        None => run_panels(),
        Some("visuals") => {
            let out = out_dir(1);
            let t0 = std::time::Instant::now();
            let mut mark = |m: &str| println!("[{:7.1}s] {m}", t0.elapsed().as_secs_f64());
            let data = visuals::build_data(&Design::new(), &mut mark).dump();
            write(&out.join("data.json"), &data);
            mark("DONE -> data.json");
            splice_pages(&data, &out);
        }
        Some("splice") => {
            let out = out_dir(1);
            splice_pages(&read(&visuals_dir().join("data.json")), &out);
        }
        Some("ts-diagram") => {
            write_ts_diagram(&Design::new());
            println!("T–s diagram data (ideal vs real) written to {TS_DIAGRAM_JSON}");
        }
        Some("panel") => {
            let name = args.get(1).map(String::as_str).unwrap_or_else(|| {
                eprintln!("usage: panel NAME [--write]");
                std::process::exit(2);
            });
            let write_it = match args.get(2).map(String::as_str) {
                None => false,
                Some("--write") => true,
                Some(other) => {
                    eprintln!("unknown option {other:?}; expected `--write`");
                    std::process::exit(2);
                }
            };
            let hits: Vec<_> = PANELS.iter().filter(|(n, _)| *n == name).collect();
            if hits.is_empty() {
                eprintln!("no panel named {name:?}");
                std::process::exit(2);
            }
            if write_it && !RUST_OWNED.contains(&name) {
                eprintln!("{name:?} is not a Rust-owned panel; only those have a capture to write");
                std::process::exit(2);
            }
            let design = Design::new();
            let mut printer = Printer::new();
            for (_, panel) in hits {
                panel(&mut printer, &design);
            }
            let text = printer.take();
            if write_it {
                let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("oracle").join("rust_owned");
                std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
                let path = dir.join(format!("{name}.txt"));
                write(&path, &text);
                println!("{} bytes -> {}", text.len(), path.display());
            } else {
                std::io::stdout().write_all(text.as_bytes()).expect("stdout closed");
            }
        }
        Some("sandbox") => {
            let wasm_path = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
                eprintln!("usage: sandbox WASM [OUT]");
                std::process::exit(2);
            });
            let dir = visuals_dir().join("..").join("sandbox");
            let out = args.get(2).map(PathBuf::from).unwrap_or_else(|| dir.join("turbojet-sandbox.html"));
            let wasm = std::fs::read(&wasm_path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", wasm_path.display()));
            let page = sandbox::splice_page(&read(&dir.join("template.html")), &wasm);
            write(&out, &page);
            println!("{} bytes ({} of model) -> {}", page.len(), wasm.len(), out.display());
        }
        Some("sandbox-native") => {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            for req in sandbox::check_requests() {
                writeln!(out, "{req}\t{}", sandbox::call(&req)).expect("stdout closed");
            }
        }
        Some(other) => {
            eprintln!("unknown subcommand {other:?}; expected none, `visuals [DIR]`, `splice [DIR]`, `ts-diagram`, `panel NAME [--write]`, `sandbox WASM [OUT]` or `sandbox-native`");
            std::process::exit(2);
        }
    }
}
