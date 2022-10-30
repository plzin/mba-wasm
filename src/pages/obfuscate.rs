use std::fmt::{self, Display, Formatter, Write};
use std::num::Wrapping;

use rand::distributions::{Standard, Distribution};
use wasm_bindgen::prelude::*;
use super::Bitness;
use crate::congruence_solver::{ModN, solve_congruences};
use crate::matrix::Matrix;
use crate::vector::Vector;
use crate::printer::Printer;
use crate::uniform_expr::{UniformNum, LUExpr, UExpr, Valuation};

#[wasm_bindgen]
pub fn obfuscate(req: ObfuscateReq) -> Result<String, String> {
    match req.bits {
        Bitness::U8 => obfuscate_impl::<Wrapping<u8>>(req),
        Bitness::U16 => obfuscate_impl::<Wrapping<u16>>(req),
        Bitness::U32 => obfuscate_impl::<Wrapping<u32>>(req),
        Bitness::U64 => obfuscate_impl::<Wrapping<u64>>(req),
        Bitness::U128 => obfuscate_impl::<Wrapping<u128>>(req),
    }
}

#[wasm_bindgen]
pub fn normalize_op(expr: String) -> String {
    UExpr::from_string(expr)
        .map_or(String::new(), |s| s.to_string())
}

fn obfuscate_impl<T: UniformNum + std::fmt::Display>(
    req: ObfuscateReq
) -> Result<String, String>
    where 
        T: UniformNum + std::fmt::Display,
        Standard: Distribution<T>
{
    let expr = LUExpr::<T>::from_string(req.expr).ok_or(
        "Input is not a linear combination of uniform expressions".to_owned()
    )?;

    rewrite(&expr, &req.ops, req.randomize)
        .map(|e| req.printer.print_luexpr(&e, req.bits))
        .ok_or("Operations can't be used to rewrite the input".to_owned())
}

fn rewrite<T: UniformNum + std::fmt::Display>(
    expr: &LUExpr<T>, ops: &[UExpr], randomize: bool
) -> Option<LUExpr<T>>
    where 
        T: UniformNum + std::fmt::Display,
        Standard: Distribution<T>
{
    // Find all variables.
    let mut v = Vec::new();
    expr.vars(&mut v);
    for op in ops {
        op.vars(&mut v);
    }

    // Remove duplicates and sort.
    v.sort();
    v.dedup();

    let mut val = Valuation::zero(&v);

    let rows = 1usize << v.len();
    let cols = ops.len();

    let mut a = Matrix::zero(rows, cols);
    let mut b = Vector::zero(rows);

    // Initialize the matrix.
    for i in 0..rows {
        let row = a.row_mut(i);

        // Initialize the valuation.
        for (j, c) in v.iter().enumerate() {
            if (i >> j) & 1 == 0 {
                val[*c] = T::zero();
            } else {
                val[*c] = T::zero() - T::one();
            }
        }

        // Write the values of the operations into this row of the matrix.
        for (j, e) in ops.iter().enumerate() {
            row[j] = e.eval(&val);
        }

        // Write the desired result into the vector.
        b[i] = expr.eval(&val);
    }

    // Solve the system.
    let l = solve_congruences(a, &b);

    // Does it have solutions?
    if l.is_empty() {
        return None;
    }

    // Sample a point from the lattice.
    let mut solution = l.offset;
    if randomize {
        for b in l.basis {
            solution += &(b * rand::random());
        }
    }

    // Put it in an LUExpr.
    let v = solution.iter()
        .zip(ops.iter())
        .map(|(m, e)| (*m, e.clone()))
        .collect();

    Some(LUExpr(v))
}

/// Obfuscation settings.
#[wasm_bindgen]
#[derive(Debug)]
pub struct ObfuscateReq {
    /// The expression to obfuscate.
    #[wasm_bindgen(skip)]
    pub expr: String,

    /// The operations used for rewriting.
    #[wasm_bindgen(skip)]
    pub ops: Vec<UExpr>,

    /// The integer width.
    pub bits: Bitness,

    /// Should the solution be randomized.
    pub randomize: bool,

    /// How to print the result.
    pub printer: Printer,
}

#[wasm_bindgen]
impl ObfuscateReq {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            expr: String::new(),
            ops: Vec::new(),
            bits: Bitness::U8,
            randomize: true,
            printer: Printer::C,
        }
    }

    #[wasm_bindgen(setter)]
    pub fn set_expr(&mut self, expr: String) {
        self.expr = expr;
    }

    #[wasm_bindgen]
    pub fn add_op(&mut self, op: String) {
        let expr = UExpr::from_string(op)
            .expect("Failed to add parse the operator");
        self.ops.push(expr);
    }

    //#[wasm_bindgen(setter)]
    //pub fn set_bits(&mut self, bits: Bitness) {
    //    self.bits = bits;
    //}

    //#[wasm_bindgen(setter)]
    //pub fn set_randomize(&mut self, randomize: bool) {
    //    self.randomize = randomize;
    //}

    //#[wasm_bindgen(setter)]
    //pub fn set_printer(&mut self, printer: Printer) {
    //    self.printer = printer;
    //}
}