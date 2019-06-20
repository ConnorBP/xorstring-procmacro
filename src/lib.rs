#![warn(rust_2018_idioms)]

extern crate proc_macro;

use proc_macro2::Span;
use quote::quote;
use std::time::{SystemTime, UNIX_EPOCH};
use syn::{parse_macro_input, Lit};

lazy_static::lazy_static! {
    // Random key, trust me!
    static ref XORKEY: u8 = xorstr_random_number(0, <u8>::max_value().into()) as u8;

    // Generate a key using the compile time
    static ref TEMP_KEY: u32 = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u32
                    & 0xFFFFFFF;
}

fn linear_congruent_generator(rounds: u32) -> u32 {
    1013904223
        + 1664525
            * if rounds <= 0 {
                *TEMP_KEY
            } else {
                linear_congruent_generator(rounds - 1)
            }
}

fn xorstr_random() -> u32 {
    linear_congruent_generator(10)
}

fn xorstr_random_number(min: u32, max: u32) -> u32 {
    min + xorstr_random() % (max - min + 1)
}

#[proc_macro]
pub fn xorstring(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if input.is_empty() {
        // fetch the key used for ALL encryptions for use in decryption
        let key = *XORKEY;
        return quote! { (#key as u8) }.into();
    }

    let string: Lit = parse_macro_input!(input as Lit);
    let string = match string {
        Lit::ByteStr(lit) => lit,
        _ => panic!("not byte string input"), // only bytes are ok right now
    };

    let mut encrypted = string.value();
    for (i, c) in encrypted.iter_mut().enumerate() {
        // XOR every character to encrypt it with the key
        *c ^= (*XORKEY as usize + i) as u8;
    }

    // ok boys it's encrypted, time to send it off to the caller!
    let lit = syn::LitByteStr::new(encrypted.as_slice(), Span::call_site());
    return quote! { (#lit as &[u8]) }.into();
}
