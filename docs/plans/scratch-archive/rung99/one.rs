// Rung 99: answer one request file, print the reason (or "ran").
fn main() {
    let r = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let out = turbojet::sandbox::call(r.trim());
    match out.find("\"reason\":\"") { Some(p) => println!("{}", &out[p + 10..out.len().min(p + 1200)]), None => println!("ran") }
}
