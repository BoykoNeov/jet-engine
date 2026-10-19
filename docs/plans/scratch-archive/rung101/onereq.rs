// Print the reply to one request read from a file (the reason, trimmed).
fn main() {
    let r = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    for l in r.lines() { let out = turbojet::sandbox::call(l); println!("{}", &out[..out.len().min(700)]); }
}
