// Print the page's replies to the requests given as arguments (sandbox::call).
fn main() {
    for a in std::env::args().skip(1) {
        println!("{a}\n  -> {}", turbojet::sandbox::call(&a));
    }
}
