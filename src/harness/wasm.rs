//! A Strategy that wraps a wasm module.

use std::{fmt::Display, fs::File, io::Read, path::Path};

use wasmer::{imports, Instance, Memory, Module, NativeFunc, Store};

use crate::{
    strategy::{Attempts, AttemptsKey, Puzzle, Word},
    HarnessError, Strategy, WordleError,
};

fn bool_to_i32(b: bool) -> i32 {
    match b {
        false => 0i32,
        true => 1i32,
    }
}

fn i32_to_bool(n: i32) -> bool {
    n == 1
}

/// This function assumes that the ptr is an i32 in an 8 byte slot and that
/// the len is an i32 in the next 8 byte slot.
fn get_wasm_bytes_and_free(
    memory: &Memory,
    free: NativeFunc<(i32, i32, i32), ()>,
    baseline: i32,
) -> Result<Vec<u8>, WordleError> {
    assert_eq!(baseline % 4, 0);
    assert_eq!(std::mem::size_of::<i32>(), 4);
    let index = (baseline / 4) as usize;

    let view = memory.view::<i32>();
    let ptr = view[index + 0].get();
    let len = view[index + 2].get();

    let view = memory.view::<u8>();
    let bytes = view[(ptr as usize)..((ptr + len) as usize)]
        .iter()
        .map(|c| c.get())
        .collect();

    free.call(ptr, len * 1, 1)
        .map_err(|e| HarnessError::Wasm(Box::new(e)))?;

    Ok(bytes)
}

#[derive(Debug, Clone)]
pub struct WasmWrapper {
    instance: Instance,
    hard: bool,
    version: String,
}

impl WasmWrapper {
    pub fn new(wasm_path: impl AsRef<Path>) -> Result<Self, WordleError> {
        let mut binary = Vec::new();
        let mut wasm_file = File::options()
            .write(false)
            .read(true)
            .open(wasm_path)
            .map_err(|e| HarnessError::Wasm(Box::new(e)))?;
        wasm_file
            .read_to_end(&mut binary)
            .map_err(|e| HarnessError::Wasm(Box::new(e)))?;

        let store = Store::default();
        let module =
            Module::from_binary(&store, &binary).map_err(|e| HarnessError::Wasm(Box::new(e)))?;
        let imports = imports! {};
        let instance =
            Instance::new(&module, &imports).map_err(|e| HarnessError::Wasm(Box::new(e)))?;

        Self::validate(instance)
    }

    fn validate(instance: Instance) -> Result<Self, WordleError> {
        // Make sure that the instance has the proper exports, etc.
        let canonical_abi_free: NativeFunc<(i32, i32, i32), ()> = instance
            .exports
            .get_native_function("canonical_abi_free")
            .map_err(|e| HarnessError::Wasm(Box::new(e)))?;
        let init_trampoline: NativeFunc<(), i32> = instance
            .exports
            .get_native_function("init_trampoline")
            .map_err(|e| HarnessError::Wasm(Box::new(e)))?;
        let _solve_trampoline: NativeFunc<(i32, i32, i32, i32), i32> = instance
            .exports
            .get_native_function("solve_trampoline")
            .map_err(|e| HarnessError::Wasm(Box::new(e)))?;
        let memory = instance
            .exports
            .get_memory("memory")
            .map_err(|e| HarnessError::Wasm(Box::new(e)))?;

        let baseline = init_trampoline
            .call()
            .map_err(|e| HarnessError::Wasm(Box::new(e)))?;

        assert_eq!(baseline % 4, 0);
        assert_eq!(std::mem::size_of::<i32>(), 4);
        let index = (baseline / 4) as usize;

        let view = memory.view::<i32>();
        let hard = i32_to_bool(view[index + 0].get());
        let bytes = get_wasm_bytes_and_free(memory, canonical_abi_free, baseline + 8)?;
        let version = String::from_utf8(bytes).map_err(|e| HarnessError::Wasm(Box::new(e)))?;

        Ok(Self {
            instance,
            hard,
            version,
        })
    }
}

impl Strategy for WasmWrapper {
    fn solve(&self, puzzle: &mut Puzzle, key: AttemptsKey) -> Attempts {
        // We checked that the following exports existed earlier, so it should
        // be okay to unwrap them now.
        let canonical_abi_free: NativeFunc<(i32, i32, i32), ()> = self
            .instance
            .exports
            .get_native_function("canonical_abi_free")
            .unwrap();
        let solve_trampoline: NativeFunc<(i32, i32, i32, i32), i32> = self
            .instance
            .exports
            .get_native_function("solve_trampoline")
            .unwrap();
        let memory = self.instance.exports.get_memory("memory").unwrap();

        let Puzzle { word, poisoned } = puzzle;
        let Word { index: word_index } = word;
        let AttemptsKey { hard, cheat } = key;

        let baseline = solve_trampoline
            .call(
                *word_index as i32,
                bool_to_i32(*poisoned),
                bool_to_i32(hard),
                bool_to_i32(cheat),
            )
            .unwrap(); // if this goes wrong, there isn't much we can do

        assert_eq!(baseline % 4, 0);
        assert_eq!(std::mem::size_of::<i32>(), 4);
        let index = (baseline / 4) as usize;

        let view = memory.view::<i32>();
        let poisoned = i32_to_bool(view[index + 0].get());
        let bytes = get_wasm_bytes_and_free(memory, canonical_abi_free, baseline + 8).unwrap();

        puzzle.poisoned = puzzle.poisoned || poisoned;
        rmp_serde::from_slice(&bytes).unwrap()
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn hardmode(&self) -> bool {
        self.hard
    }
}

impl Display for WasmWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Fix this...
        write!(f, "wrapped: get name!")
    }
}

#[cfg(test)]
mod test {
    use super::{bool_to_i32, i32_to_bool};

    #[test]
    fn bool_i32_conversion() {
        assert_eq!(true, i32_to_bool(bool_to_i32(true)));
        assert_eq!(false, i32_to_bool(bool_to_i32(false)));
    }
}
