#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use proc_macro;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, DeriveInput, Token,
};

struct WrappableArgs {
    init: (kw::new, Token![=], Ident),
    _separator: Token![,],
    name: (kw::name, Token![=], Ident),
}

impl Parse for WrappableArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::new) {
            Ok(Self {
                init: (input.parse::<kw::new>()?, input.parse()?, input.parse()?),
                _separator: input.parse()?,
                name: (input.parse::<kw::name>()?, input.parse()?, input.parse()?),
            })
        } else if lookahead.peek(kw::name) {
            Ok(Self {
                name: (input.parse::<kw::name>()?, input.parse()?, input.parse()?),
                _separator: input.parse()?,
                init: (input.parse::<kw::new>()?, input.parse()?, input.parse()?),
            })
        } else {
            Err(lookahead.error())
        }
    }
}

/// A derive-like attribute for Wordle strategies written with `wordle_rs`
/// that emits wasm entry points for use with that crate's `WasmWrapper` type.
///
/// # How to use
///
/// The attribute takes two arguments: an unambiguous name (within your
/// crate) for the strategy and an associated method that can initialize it.
/// These arguments are passed by `[key] = [value]` (see example below).
///
/// Each has some requirements:
///
/// - `name`: This must be a valid Rust identifier. Something like
///   `my_cool_strategy` will work; `My Cool Strategy` will not.
/// - `new`: This must be a const associated function defined on the strategy type
///   that *takes no arguments*. Eventually this requirement may change.
///
/// # Examples
///
/// ```ignore
/// use wordle_rs::{Strategy, wrappable};
///
/// #[wrappable(name = my_cool_strategy, new = new)]
/// pub struct MyCoolStrategy {
///     data: u8
/// }
///
/// impl MyCoolStrategy {
///     pub const fn new() -> Self {
///         Self { data: 0_u8 }    
///     }
/// }
///
/// impl Strategy for MyCoolStrategy {
///     // snip
/// }
/// ```
#[proc_macro_attribute]
pub fn wrappable(
    attr: proc_macro::TokenStream,
    mut item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // TODO: figure out how to parse attr
    let options = parse_macro_input!(attr as WrappableArgs);
    let input = item.clone();
    let input = parse_macro_input!(input as DeriveInput);

    proc_macro::TokenStream::from(wrappable_inner(options, input))
}

fn wrappable_inner(options: WrappableArgs, input: DeriveInput) -> TokenStream {
    let mod_name = format_ident!("__{}_wasm_hooks", options.name.2);
    let init_trampoline_name = format_ident!("__init_trampoline_{}", options.name.2);
    let solve_trampoline_name = format_ident!("__solve_trampoline_{}", options.name.2);
    let exported_init_name = init_trampoline_name.to_string();
    let exported_solve_name = solve_trampoline_name.to_string();

    let struct_name = &input.ident;
    let struct_init = &options.init.2;

    let init_function = quote! {
        fn #init_trampoline_name() -> (bool, String) {
            (THIS.hardmode(), THIS.version().to_string())
        }
    };

    let solve_function = quote! {
        fn #solve_trampoline_name(mut puzzle: Puzzle, key: AttemptsKey) -> (bool, Vec<u8>) {
            let attempts = THIS.solve(&mut puzzle, key);
            let buf = rmp_serde::to_vec(&attempts).unwrap();
            (puzzle.__secret_is_poisoned(), buf)
        }
    };

    let glue = quote! {
        // TODO: change name
        #[export_name = #exported_init_name]
        unsafe extern "C" fn __wit_bindgen_init_trampoline() -> i32 {
            let (result0_0, result0_1) = #init_trampoline_name();
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

        // TODO: change name
        #[export_name = #exported_solve_name]
        unsafe extern "C" fn __wit_bindgen_solve_trampoline(
            word_index: i32,
            poisoned: i32,
            hard: i32,
            cheat: i32,
        ) -> i32 {
            let (new_poisoned, attempts_mp) = #solve_trampoline_name(
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
        static mut RET_AREA: [i64; 3] = [0; 3];
    };

    quote! {
        #input

        #[cfg(target_family = "wasm")]
        mod #mod_name {
            #![allow(missing_docs)]

            // TODO: fix
            use wordle_rs::strategy::{AttemptsKey, Puzzle, Strategy, Word};

            use super::*;

            // TODO: call init instead of new
            static THIS: #struct_name = <#struct_name>::#struct_init();

            #init_function

            #solve_function

            #glue
        }
    }
}

mod kw {
    syn::custom_keyword!(new);
    syn::custom_keyword!(name);
}
