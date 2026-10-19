use turbojet::sandbox::call;
fn main() {
    let text = std::fs::read_to_string("W:/temp/claude/idle-rule/offmap_requests.txt").unwrap();
    for l in text.lines() { println!("{}\n", call(l)); }
}
