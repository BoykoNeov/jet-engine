// Rung 98 census: every Controls check request through this build, one line each: index, the key hash of what it did.
#[path = "crash98.rs"]
#[allow(dead_code)]
mod crash;
fn main() {
    for (i, r) in turbojet::sandbox_controls::check_requests().iter().enumerate() {
        println!("{i}\t{:016x}\t{r}", crash::key_hash(&turbojet::sandbox::call(r)));
    }
}
