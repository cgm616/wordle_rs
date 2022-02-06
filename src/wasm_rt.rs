use std::alloc::{self, Layout};

// Taken from https://github.com/bytecodealliance/wit-bindgen/blob/d505f87e67c42006631f913152ab16f96153cef4/crates/rust-wasm/src/futures.rs
#[no_mangle]
unsafe extern "C" fn canonical_abi_realloc(
    old_ptr: *mut u8,
    old_len: usize,
    align: usize,
    new_len: usize,
) -> *mut u8 {
    let layout;
    let ptr = if old_len == 0 {
        if new_len == 0 {
            return align as *mut u8;
        }
        layout = Layout::from_size_align_unchecked(new_len, align);
        alloc::alloc(layout)
    } else {
        layout = Layout::from_size_align_unchecked(old_len, align);
        alloc::realloc(old_ptr, layout, new_len)
    };
    if ptr.is_null() {
        alloc::handle_alloc_error(layout);
    }
    return ptr;
}

// Taken from https://github.com/bytecodealliance/wit-bindgen/blob/d505f87e67c42006631f913152ab16f96153cef4/crates/rust-wasm/src/futures.rs
#[no_mangle]
unsafe extern "C" fn canonical_abi_free(ptr: *mut u8, len: usize, align: usize) {
    if len == 0 {
        return;
    }
    let layout = Layout::from_size_align_unchecked(len, align);
    alloc::dealloc(ptr, layout);
}
