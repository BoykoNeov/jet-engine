// Read request lines from a file, print `REQUEST<TAB>REPLY` (a panic becomes `PANIC: message`).
use std::io::{BufRead, Write};
fn main() {
    let path = std::env::args().nth(1).expect("requests file");
    std::panic::set_hook(Box::new(|_| {}));
    let out = std::io::stdout();
    let mut out = out.lock();
    for line in std::io::BufReader::new(std::fs::File::open(path).unwrap()).lines() {
        let req = line.unwrap();
        let reply = std::panic::catch_unwind(|| turbojet::sandbox::call(&req)).unwrap_or_else(|e| {
            format!("PANIC: {}", e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default())
        });
        writeln!(out, "{req}\t{reply}").unwrap();
    }
}
