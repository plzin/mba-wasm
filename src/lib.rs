#![allow(unused)]
#![feature(min_specialization)]

mod vector;
mod matrix;
mod numbers;
mod polynomial;
mod congruence_solver;
mod uniform_expr;
mod printer;
mod pages;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}
