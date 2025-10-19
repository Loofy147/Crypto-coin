use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn get_storage(key_ptr: *const u8, key_len: usize, value_ptr: *mut u8, value_len: usize) -> usize;
    fn set_storage(key_ptr: *const u8, key_len: usize, value_ptr: *const u8, value_len: usize);
}

#[wasm_bindgen]
pub fn increment() {
    let key = "count";
    let mut value_buf = [0u8; 8];
    let value_len = unsafe { get_storage(key.as_ptr(), key.len(), value_buf.as_mut_ptr(), value_buf.len()) };
    let mut count = if value_len > 0 {
        u64::from_le_bytes(value_buf)
    } else {
        0
    };
    count += 1;
    unsafe { set_storage(key.as_ptr(), key.len(), count.to_le_bytes().as_ptr(), 8) };
}

#[wasm_bindgen]
pub fn get_count() -> u64 {
    let key = "count";
    let mut value_buf = [0u8; 8];
    let value_len = unsafe { get_storage(key.as_ptr(), key.len(), value_buf.as_mut_ptr(), value_buf.len()) };
    if value_len > 0 {
        u64::from_le_bytes(value_buf)
    } else {
        0
    }
}
