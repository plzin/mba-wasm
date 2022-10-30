//! API for webpages.

mod obfuscate;
mod linear_congruences;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum Bitness {
    U8,
    U16,
    U32,
    U64,
    U128,
}

pub fn underbrace<T: AsRef<str>, U: AsRef<str>>(inner: T, label: U) -> String {
    format!("\\underbrace{{{}}}_{{{}}}", inner.as_ref(), label.as_ref())
}

pub fn bold<T: AsRef<str>>(inner: T) -> String {
    format!("\\mathbf{{{}}}", inner.as_ref())
}