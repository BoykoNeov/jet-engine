//! The browser shell around `turbojet::sandbox::call` (docs/plans/sandbox-plan.md § 2–3).
//!
//! Text crosses the boundary as UTF-8 bytes in the module's memory: JS asks for a buffer
//! (`alloc`), writes the request into it, calls `call`, and reads the reply at `reply_ptr` /
//! `reply_len`. A design that fails INSIDE the model panics; in wasm32 a panic is a trap, which JS
//! sees as a `RuntimeError`. The hook below has already stored the message by then, so JS reads it
//! at `panic_ptr` / `panic_len` — and then throws this instance away and makes a fresh one, because
//! a trap can leave the allocator or a lock half-updated.

use std::sync::Mutex;

static REPLY: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static PANIC: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// Install the panic hook. Call once per instance, before anything else.
#[no_mangle]
pub extern "C" fn init() {
    std::panic::set_hook(Box::new(|info| {
        let text = match info.payload().downcast_ref::<String>() {
            Some(s) => s.clone(),
            None => info.payload().downcast_ref::<&str>().map(|s| s.to_string()).unwrap_or_default(),
        };
        if let Ok(mut m) = PANIC.lock() {
            *m = text.into_bytes();
        }
    }));
}

/// A buffer of `len` bytes for JS to write a request into.
#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len.max(1));
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// Give back a buffer from [`alloc`].
///
/// # Safety
/// `ptr` and `len` must be exactly what one earlier `alloc(len)` returned and was given.
#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut u8, len: usize) {
    drop(Vec::from_raw_parts(ptr, 0, len.max(1)));
}

/// Answer the request in `[ptr, ptr+len)`; the reply is then at [`reply_ptr`] / [`reply_len`].
///
/// # Safety
/// The range must be a buffer from [`alloc`] holding `len` bytes of UTF-8 text.
#[no_mangle]
pub unsafe extern "C" fn call(ptr: *const u8, len: usize) {
    let request = std::str::from_utf8(std::slice::from_raw_parts(ptr, len)).expect("request is not UTF-8");
    let reply = turbojet::sandbox::call(request);
    *REPLY.lock().unwrap() = reply.into_bytes();
}

#[no_mangle]
pub extern "C" fn reply_ptr() -> *const u8 { REPLY.lock().map(|r| r.as_ptr()).unwrap_or(std::ptr::null()) }

#[no_mangle]
pub extern "C" fn reply_len() -> usize { REPLY.lock().map(|r| r.len()).unwrap_or(0) }

#[no_mangle]
pub extern "C" fn panic_ptr() -> *const u8 { PANIC.lock().map(|r| r.as_ptr()).unwrap_or(std::ptr::null()) }

#[no_mangle]
pub extern "C" fn panic_len() -> usize { PANIC.lock().map(|r| r.len()).unwrap_or(0) }
