use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter, Write};
use std::num::Wrapping;
use std::rc::Rc;
use rand::distributions::{Standard, Distribution};
use wasm_bindgen::prelude::*;
use super::Width;
use crate::congruence_solver::solve_congruences;
use crate::matrix::Matrix;
use crate::vector::Vector;
use crate::printer::Printer;
use crate::expr::Expr;
use crate::uniform_expr::{LUExpr, UExpr, Valuation};
use crate::numbers::{UnsignedInt, UniformNum};

#[wasm_bindgen]
pub fn obfuscate(
    expr: String, width: Width, out: Printer
) -> Result<String, String> {
    match width {
        Width::U8   => obfuscate_impl::<Wrapping<u8>>(expr, out),
        Width::U16  => obfuscate_impl::<Wrapping<u16>>(expr, out),
        Width::U32  => obfuscate_impl::<Wrapping<u32>>(expr, out),
        Width::U64  => obfuscate_impl::<Wrapping<u64>>(expr, out),
        Width::U128 => obfuscate_impl::<Wrapping<u128>>(expr, out),
    }
}

fn obfuscate_impl<T: UniformNum + std::fmt::Debug>(
    expr: String, out: Printer
) -> Result<String, String> 
    where Standard: Distribution<T>
{
    let mut e = Rc::new(Expr::<T>::from_string(expr)?);
    crate::log(&format!("{:?}", e));

    let mut vars = e.vars();
    for i in 0..(REWRITE_VARS - vars.len() as isize) {
        vars.push(format!("aux{}", i));
    }

    let mut v = Vec::new();
    obfuscate_expr(&mut e, &mut v, &vars);
    Ok(e.print_as_fn(out))
}

/// Tries to convert the expression to a uniform expression.
/// When part of the expression isn't a uniform expression,
/// it generates a variable and remembers what expression to
/// substitute for that variable.
fn expr_to_uexpr<T: UniformNum>(
    e: &Rc<Expr<T>>, subs: &mut Vec<(String, Rc<Expr<T>>)>
) -> UExpr {
    if Rc::strong_count(e) > 1 {
        let var = format!("_sub_{}", subs.len());
        subs.push((var.clone(), e.clone()));
        return UExpr::Var(var);
    }

    match e.as_ref() {
        Expr::Var(v) => UExpr::Var(v.clone()),
        Expr::And(l, r) => UExpr::and(expr_to_uexpr(l, subs), expr_to_uexpr(r, subs)),
        Expr::Or(l, r) => UExpr::or(expr_to_uexpr(l, subs), expr_to_uexpr(r, subs)),
        Expr::Xor(l, r) => UExpr::xor(expr_to_uexpr(l, subs), expr_to_uexpr(r, subs)),
        Expr::Not(i) => UExpr::not(expr_to_uexpr(i, subs)),
        // Otherwise generate a new variable and add the substitution.
        _ => {
            let var = format!("_sub_{}", subs.len());
            subs.push((var.clone(), e.clone()));
            UExpr::Var(var)
        }
    }
}

/// Tries to convert an expression
fn parse_term<T: UniformNum>(
    e: &Rc<Expr<T>>, subs: &mut Vec<(String, Rc<Expr<T>>)>
) -> (T, UExpr) {
    if let Expr::Mul(l, r) = e.as_ref() {
        if let Expr::Const(i) = l.as_ref() {
            return (*i, expr_to_uexpr(r, subs));
        } else if let Expr::Const(i) = r.as_ref() {
            return (*i, expr_to_uexpr(l, subs));
        }
    } else if let Expr::Const(c) = e.as_ref() {
        return (T::zero() - *c, UExpr::Ones);
    }

    (T::one(), expr_to_uexpr(e, subs))
}

fn expr_to_luexpr<T: UniformNum>(
    e: &Rc<Expr<T>>,
    lu: &mut LUExpr<T>,
    subs: &mut Vec<(String, Rc<Expr<T>>)>,
    sign: bool
) {
    // If this is an add the left and right hand side
    // can contribute to the linear combination.
    match e.as_ref() {
        Expr::Add(l, r) => {
            expr_to_luexpr(l, lu, subs, sign);
            expr_to_luexpr(r, lu, subs, sign);
        },

        Expr::Sub(l, r) => {
            expr_to_luexpr(l, lu, subs, sign);
            expr_to_luexpr(r, lu, subs, !sign);
        },

        Expr::Neg(i) => {
            // Theoretically we could allow another whole
            // LUExpr in here but hopefully not too important.

            // Flipped because of the Neg.
            let f = if sign { T::one() } else { T::zero() - T::one() };
            lu.0.push((f, expr_to_uexpr(i, subs)));
        },

        // Otherwise parse the term from this expression.
        _ => {
            let (mut f, u) = parse_term(e, subs);
            if sign {
                f = T::zero() - f;
            }
            lu.0.push((f, u));
        },
    }
}

fn obfuscate_expr<T: UniformNum>(er: &mut Rc<Expr<T>>, visited: &mut Vec<*const Expr<T>>, vars: &[String])
    where Standard: Distribution<T>
{
    let ptr = Rc::as_ptr(er);
    if Rc::strong_count(er) > 1 {
        if visited.contains(&ptr) {
            return;
        }
        visited.push(ptr);
    }

    let e = unsafe { &mut *(ptr as *mut _) };

    match e {
        Expr::Mul(l, r) => {
            obfuscate_expr(l, visited, vars);
            obfuscate_expr(r, visited, vars);
        },
        Expr::Div(l, r) | Expr::Mod(l, r) => {
            obfuscate_expr(l, visited, vars);
            obfuscate_expr(r, visited, vars);
        },
        Expr::Shl(l, r) | Expr::Shr(l, r) => {
            obfuscate_expr(l, visited, vars);
            obfuscate_expr(r, visited, vars);
        },
        _ => {
            // Try to find the largest subexpression that is linear MBA
            // and obfuscate it on its own.
            let mut lu = LUExpr(Vec::new());

            // Substitutions in the LUExpr.
            let mut subs: Vec<(String, Rc<Expr<T>>)> = Vec::new();

            expr_to_luexpr(er, &mut lu, &mut subs, false);
            crate::log(&Printer::C.print_luexpr(&lu));
            *e = rewrite_random(&lu, vars).to_expr();
            crate::log(&e.print_as_fn(Printer::C));
            for (var, sub) in &mut subs {
                // Obfuscate the substituted expressions.
                obfuscate_expr(sub, visited, vars);

                // Substitute them for the variables.
                e.substitute(sub, var);
            }
        }
    }
}

const REWRITE_VARS: isize = 4;
const REWRITE_EXPR_DEPTH: u8 = 3;
const REWRITE_EXPR_COUNT: usize = 24;
const REWRITE_TRIES: usize = 128;

fn rewrite_random<T: UniformNum>(e: &LUExpr<T>, vars: &[String]) -> LUExpr<T>
    where Standard: Distribution<T>
{
    let mut vars: Vec<_> = vars.iter().cloned().collect();
    for v in e.vars() {
        if !vars.contains(&v) {
            vars.push(v);
        }
    }
    for _ in 0..REWRITE_TRIES {
        let mut ops = Vec::new();
        for _ in 0..REWRITE_EXPR_COUNT {
            ops.push(LUExpr::from_uexpr(
                random_bool_expr(&vars, REWRITE_EXPR_DEPTH)
            ));
        }

        if let Some(r) = rewrite(e, &ops, true) {
            return r;
        }
    }

    panic!("Failed to rewrite uniform expression.");
}

/// Note that this never generates `Ones` or any expression containing it,
/// as those can be easily simplified to one that does not contain it.
fn random_bool_expr<T: AsRef<str>>(vars: &[T], max_depth: u8) -> UExpr {
    assert!(!vars.is_empty(), "There needs to be at least one variable for the random expression.");

    let rand_var = || UExpr::Var(vars[rand::random::<usize>() % vars.len()].as_ref().to_owned());

    if max_depth == 0 {
        return rand_var();
    }

    // Generate one of the four variants uniformly at random.
    let d = max_depth - 1;
    match rand::random::<usize>() % 5 {
        0 => rand_var(),
        1 => UExpr::Not(random_bool_expr(vars, d).into()),
        2 => UExpr::And(random_bool_expr(vars, d).into(), random_bool_expr(vars, d).into()),
        3 => UExpr::Or(random_bool_expr(vars, d).into(), random_bool_expr(vars, d).into()),
        4 => UExpr::Xor(random_bool_expr(vars, d).into(), random_bool_expr(vars, d).into()),
        _ => unreachable!(),
    }
}

#[wasm_bindgen]
pub fn obfuscate_linear(req: ObfLinReq) -> Result<String, String> {
    match req.bits {
        Width::U8   => obfuscate_linear_impl::<Wrapping<u8>>(req),
        Width::U16  => obfuscate_linear_impl::<Wrapping<u16>>(req),
        Width::U32  => obfuscate_linear_impl::<Wrapping<u32>>(req),
        Width::U64  => obfuscate_linear_impl::<Wrapping<u64>>(req),
        Width::U128 => obfuscate_linear_impl::<Wrapping<u128>>(req),
    }
}

#[wasm_bindgen]
pub fn normalize_op(expr: String, bits: Width) -> String {
    match bits {
        Width::U8 => LUExpr::<Wrapping<u8>>::from_string(expr)
            .map_or(String::new(), |s| s.to_string()),
        Width::U16 => LUExpr::<Wrapping<u16>>::from_string(expr)
            .map_or(String::new(), |s| s.to_string()),
        Width::U32 => LUExpr::<Wrapping<u32>>::from_string(expr)
            .map_or(String::new(), |s| s.to_string()),
        Width::U64 => LUExpr::<Wrapping<u64>>::from_string(expr)
            .map_or(String::new(), |s| s.to_string()),
        Width::U128 => LUExpr::<Wrapping<u128>>::from_string(expr)
            .map_or(String::new(), |s| s.to_string()),
    }
}

fn obfuscate_linear_impl<T: UniformNum + std::fmt::Display>(
    req: ObfLinReq
) -> Result<String, String>
    where 
        T: UniformNum + std::fmt::Display,
        Standard: Distribution<T>
{
    let expr = LUExpr::<T>::from_string(req.expr).ok_or(
        "Input is not a linear combination of uniform expressions".to_owned()
    )?;

    let ops: Vec<_> = req.ops.into_iter()
        .map(|s| LUExpr::<T>::from_string(s).unwrap())
        .collect();

    rewrite(&expr, &ops, req.randomize)
        .map(|e| req.printer.print_luexpr(&e))
        .ok_or("Operations can't be used to rewrite the input".to_owned())
}

fn rewrite<T: UniformNum + std::fmt::Display>(
    expr: &LUExpr<T>, ops: &[LUExpr<T>], randomize: bool
) -> Option<LUExpr<T>>
    where 
        T: UniformNum + std::fmt::Display,
        Standard: Distribution<T>
{
    // Find all variables.
    let mut v = BTreeSet::new();
    expr.vars_impl(&mut v);
    for op in ops {
        op.vars_impl(&mut v);
    }

    let v: Vec<_> = v.into_iter().collect();

    let mut val = Valuation::zero(v.clone());

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
                val[c] = T::zero();
            } else {
                val[c] = T::zero() - T::one();
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
    // Currently, this simplifies the inner LUExprs into
    // sums of UExprs, such that the result is an LUExpr.
    // Once there is a more general Expr class, we need not do this.
    let mut v = Vec::new();
    for (c, o) in solution.iter().zip(ops.iter()) {
        for (d, e) in &o.0 {
            // Is the UExpr already in the linear combination?
            match v.iter_mut().find(|(_, f)| f == e) {
                Some((f, _)) => *f += *c * *d,
                None => v.push((*c * *d, e.clone())),
            }
        }
    }

    // Remove terms where the coefficient is zero.
    v.retain(|(f, u)| !f.is_zero());

    Some(LUExpr(v))
}

/// Obfuscation settings.
#[wasm_bindgen]
#[derive(Debug)]
pub struct ObfLinReq {
    /// The expression to obfuscate.
    #[wasm_bindgen(skip)]
    pub expr: String,

    /// The operations used for rewriting.
    /// There is currently an issue with this because we verify the ops
    /// with a certain bitness but the obfuscation may happen with another one.
    /// This is only really a problem with big constants though, so not that
    /// likely to happen to anyone.
    #[wasm_bindgen(skip)]
    pub ops: Vec<String>,

    /// The integer width.
    pub bits: Width,

    /// Should the solution be randomized.
    pub randomize: bool,

    /// How to print the result.
    pub printer: Printer,
}

#[wasm_bindgen]
impl ObfLinReq {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            expr: String::new(),
            ops: Vec::new(),
            bits: Width::U8,
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
        self.ops.push(op);
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