use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use wordle_rs::strategy::{Attempts, AttemptsKey, Puzzle, Strategy, Word};

use crate::Basic;

static THIS: Basic = Basic::new();

struct Input;

impl input::Input for Input {
    fn init_trampoline() -> (bool, String) {
        (THIS.hardmode(), THIS.version().to_string())
    }

    fn solve_trampoline(mut puzzle: Puzzle, key: AttemptsKey) -> (bool, Vec<u8>) {
        let attempts = THIS.solve(&mut puzzle, key);
        let buf = rmp_serde::to_vec(&attempts).unwrap();
        (puzzle.__secret_is_poisoned(), buf)
    }
}

mod input {
    use std::alloc::{self, Layout};
    use wordle_rs::strategy::{AttemptsKey, Puzzle, Word};

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

    #[export_name = "init_trampoline"]
    unsafe extern "C" fn __wit_bindgen_init_trampoline() -> i32 {
        let (result0_0, result0_1) = <super::Input as Input>::init_trampoline();
        let result1 = match result0_0 {
            false => 0i32,
            true => 1i32,
        };
        let vec2 = (result0_1.into_bytes()).into_boxed_slice();
        let ptr2 = vec2.as_ptr() as i32;
        let len2 = vec2.len() as i32;
        core::mem::forget(vec2);
        let ptr3 = RET_AREA.as_mut_ptr() as i32;
        *((ptr3 + 16) as *mut i32) = len2;
        *((ptr3 + 8) as *mut i32) = ptr2;
        *((ptr3 + 0) as *mut i32) = result1;
        ptr3
    }

    #[export_name = "solve_trampoline"]
    unsafe extern "C" fn __wit_bindgen_solve_trampoline(
        word_index: i32,
        poisoned: i32,
        hard: i32,
        cheat: i32,
    ) -> i32 {
        let (new_poisoned, attempts_mp) = <super::Input as Input>::solve_trampoline(
            Puzzle::__secret_new(
                Word::from_index(word_index as usize).unwrap(),
                poisoned == 1,
            ),
            AttemptsKey::__secret_new(hard == 1, cheat == 1),
        );
        let result1 = match new_poisoned {
            false => 0_i32,
            true => 1_i32,
        };
        let vec2 = (attempts_mp).into_boxed_slice();
        let ptr2 = vec2.as_ptr() as i32;
        let len2 = vec2.len() as i32;
        core::mem::forget(vec2);
        let ptr3 = RET_AREA.as_mut_ptr() as i32;
        *((ptr3 + 16) as *mut i32) = len2;
        *((ptr3 + 8) as *mut i32) = ptr2;
        *((ptr3 + 0) as *mut i32) = result1;
        ptr3
    }
    pub trait Input {
        fn init_trampoline() -> (bool, String);
        fn solve_trampoline(puzzle: Puzzle, key: AttemptsKey) -> (bool, Vec<u8>);
    }
    static mut RET_AREA: [i64; 3] = [0; 3];
}
