//! A Strategy that wraps a wasm module.

use std::{fmt::Display, fs::File, io::Read, path::Path};

use wasmer::{imports, Instance, Module, Store};

use crate::{
    strategy::{Attempts, AttemptsKey, Puzzle, Word},
    HarnessError, Strategy, WordleError,
};

#[derive(Debug, Clone)]
pub(crate) struct Wrapper {
    instance: Instance,
}

impl Wrapper {
    pub(crate) fn new(wasm_path: impl AsRef<Path>) -> Result<Self, WordleError> {
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
        todo!();

        Ok(Self { instance })
    }
}

impl Strategy for Wrapper {
    fn solve(&self, puzzle: &mut Puzzle, key: AttemptsKey) -> Attempts {
        // Pass to wrapped instance!
        todo!()
    }

    fn version(&self) -> &'static str {
        // Pass to wrapped instance!
        todo!()
    }

    fn hardmode(&self) -> bool {
        // Pass to wrapped instance!
        todo!()
    }
}

impl Display for Wrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Fix this...
        write!(f, "wrapped: get name!")
    }
}
