//! Print one request's reply (JSON) for inspection.
fn main() { println!("{}", turbojet::sandbox::call(&std::env::args().nth(1).unwrap())); }
