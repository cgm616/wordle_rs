//! A Strategy that wraps a wasm module.

use std::{
    fmt::Display,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use log::{debug, error, info, trace, warn};
use serde_json::Value;
use wasmer::{imports, Instance, Memory, Module, NativeFunc, Store};

use crate::{
    strategy::{Attempts, AttemptsKey, Puzzle, Word},
    HarnessError, Result, Strategy, WordleError,
};

const FREE_FUNC: &'static str = "canonical_abi_free";
const REALLOC_FUNC: &'static str = "canonical_abi_realloc";

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
) -> Result<Vec<u8>> {
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

/// A special strategy that actually wraps a wasm module compiled with
/// `wordle_rs` that defines a strategy.
///
/// This strategy simply forwards calls to the wrapped strategy.
#[derive(Debug, Clone)]
pub struct WasmWrapper {
    instance: Instance,
    short_name: String,
    solve_func: String,
    init_func: String,
    hard: bool,
    version: String,
}

impl WasmWrapper {
    /// Compiles a wasm module from a Rust library crate that defines a
    /// strategy using the `wrappable` macro.
    ///
    /// This function simply runs `cargo` on the library targeting
    /// `wasm32-unknown-unknown`, grabs the resulting wasm binary, and
    /// passes it to [`new_from_wasm()`][Self::new_from_wasm()].
    pub fn new_from_crate(crate_path: impl AsRef<Path>, short_name: &str) -> Result<Self> {
        debug!(
            "compiling crate at {:?} to wasm module",
            crate_path.as_ref()
        );

        let mut canonical =
            std::fs::canonicalize(&crate_path).map_err(|e| HarnessError::Wasm(Box::new(e)))?;

        info!("running `cargo rustc` in directory {:?}", &canonical);

        let rustc = Command::new("cargo")
            .current_dir(&canonical)
            .env("CARGO_BUILD_PIPELINING", "false")
            .args([
                "rustc",
                "--release",
                "--target=wasm32-unknown-unknown",
                "--",
                "--crate-type=cdylib",
                "--print=file-names",
            ])
            .output()
            .map_err(|e| HarnessError::Wasm(Box::new(e)))?;

        if !rustc.status.success() {
            error!("`cargo rustc` returned an error exit code!");
            match String::from_utf8(rustc.stderr) {
                Ok(s) => eprintln!("{}", s),
                Err(e) => {
                    warn!(
                        "could not print corrupted `cargo rustc` error output: {}",
                        e
                    );
                }
            }
            return Err(HarnessError::Cargo.into());
        }

        let rustc_names = String::from_utf8(rustc.stdout).map_err(|e| {
            error!("`cargo rustc` file names not valid utf8: {}", e);
            HarnessError::Cargo
        })?;

        let artifact = rustc_names
            .lines()
            .filter(|s| s.ends_with(".wasm"))
            .next()
            .ok_or_else(|| {
                error!("`cargo rustc` did not produce wasm artifact");
                HarnessError::Cargo
            })?;

        debug!("`cargo rustc` succeeded with artifact name: {}", artifact);

        debug!("running `cargo metadata`...");

        let metadata = Command::new("cargo")
            .current_dir(&canonical)
            .args(["metadata", "--format-version=1"])
            .output()
            .map_err(|e| HarnessError::Wasm(Box::new(e)))?;

        if !metadata.status.success() {
            error!("`cargo metadata` returned an error exit code!");
            return Err(HarnessError::Cargo.into());
        }

        let metadata =
            String::from_utf8(metadata.stdout).map_err(|e| HarnessError::Wasm(Box::new(e)))?;
        let json: Value =
            serde_json::from_str(&metadata).map_err(|e| HarnessError::Wasm(Box::new(e)))?;
        let target_dir = json["target_directory"]
            .as_str()
            .ok_or(HarnessError::Cargo)?;

        debug!("found target directory: {}", target_dir);

        let mut wasm_path = PathBuf::new();
        wasm_path.push(target_dir);
        wasm_path.push("wasm32-unknown-unknown/release/deps");
        wasm_path.push(artifact);
        wasm_path.set_extension("wasm");

        if !wasm_path.is_file() {
            error!("output wasm file does not exist! checked {:?}", &wasm_path);
            return Err(HarnessError::Cargo.into());
        }

        info!("building wasm module from source succeeded!");

        Self::new_from_wasm(wasm_path, short_name)
    }

    /// Wraps an already-compiled wasm module.
    ///
    /// The `wasm_path` parameter should point to a wasm binary
    /// compiled with the target `wasm32-unknown-unknown`, the `cdylib`
    /// crate type, and linked against `wordle_rs`.
    pub fn new_from_wasm(wasm_path: impl AsRef<Path>, short_name: &str) -> Result<Self> {
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

        Self::validate(instance, short_name)
    }

    fn validate(instance: Instance, short_name: &str) -> Result<Self> {
        let solve_func = format!("__solve_trampoline_{}", short_name);
        let init_func = format!("__init_trampoline_{}", short_name);

        // Make sure that the instance has the proper exports, etc.
        let canonical_abi_free: NativeFunc<(i32, i32, i32), ()> = instance
            .exports
            .get_native_function(FREE_FUNC)
            .map_err(|e| HarnessError::Wasm(Box::new(e)))?;
        let init_trampoline: NativeFunc<(), i32> = instance
            .exports
            .get_native_function(&init_func)
            .map_err(|e| HarnessError::Wasm(Box::new(e)))?;
        let _solve_trampoline: NativeFunc<(i32, i32, i32, i32), i32> = instance
            .exports
            .get_native_function(&solve_func)
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
            short_name: short_name.to_string(),
            solve_func,
            init_func,
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
            .get_native_function(FREE_FUNC)
            .unwrap();
        let solve_trampoline: NativeFunc<(i32, i32, i32, i32), i32> = self
            .instance
            .exports
            .get_native_function(&self.solve_func)
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
        let poisoned = i32_to_bool(view[index].get());
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
