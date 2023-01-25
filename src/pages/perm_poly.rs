use std::num::Wrapping;
use std::ops::ShlAssign;

use num_traits::{Num, NumAssign};
use rand::Rng;
use rand::distributions::{Standard, Distribution, Uniform};
use wasm_bindgen::prelude::*;

use crate::congruence_solver;
use crate::vector::Vector;
use crate::matrix::Matrix;
use crate::polynomial::Polynomial;
use crate::numbers::UniformNum;

use super::Bitness;

#[wasm_bindgen]
pub fn invert_poly(
    poly: String, bits: Bitness, alg: String
) -> Result<String, String> {
    match bits {
        Bitness::U8 => invert_poly_impl::<Wrapping<u8>>(poly, alg),
        Bitness::U16 => invert_poly_impl::<Wrapping<u16>>(poly, alg),
        Bitness::U32 => invert_poly_impl::<Wrapping<u32>>(poly, alg),
        Bitness::U64 => invert_poly_impl::<Wrapping<u64>>(poly, alg),
        Bitness::U128 => invert_poly_impl::<Wrapping<u128>>(poly, alg),
    }
}

#[wasm_bindgen]
pub fn rand_poly(bits: Bitness) -> String {
    match bits {
        Bitness::U8     => rand_poly_impl::<Wrapping<u8>>(),
        Bitness::U16    => rand_poly_impl::<Wrapping<u16>>(),
        Bitness::U32    => rand_poly_impl::<Wrapping<u32>>(),
        Bitness::U64    => rand_poly_impl::<Wrapping<u64>>(),
        Bitness::U128   => rand_poly_impl::<Wrapping<u128>>(),
    }
}

fn invert_poly_impl<T: UniformNum>(
    poly: String, alg: String
) -> Result<String, String> {
    // Parse the polynomial and make sure it's a permutation.
    let p = parse_poly::<T>(poly)?;
    if !is_perm_poly(&p) {
        return Err("The input is not a permutation polynomial".into());
    }

    // Find the generators of the "zero ideal".
    let zi = ZeroIdeal::<T>::init();

    // Simplify the polynomial.
    let p = p.simplified(&zi);

    // Select the algorithm.
    let alg = match alg.as_str() {
        "Newton" => invert_newton,
        "Fermat" => invert_fermat,
        "Lagrange" => invert_lagrange,
        _ => return Err("Invalid algorithm.".into()),
    };

    // Set up timing.
    let perf = web_sys::window().unwrap().performance().unwrap();
    let now = perf.now();

    // Compute the inverse.
    let q = alg(&p, &zi);

    // Log the execution time.
    let dur = perf.now() - now;
    crate::log(&format!("Inverting took {} ms", dur as u64));

    // Check that we did indeed find the inverse.
    if !compose(&p, &q, &zi).simplified(&zi).is_id() {
        crate::log("Inverse is wrong!");
    }

    // Return the inverse's tex.
    Ok(q.to_tex())
}

/// Invert using p as a generator.
fn invert_fermat<T: UniformNum>(
    p: &Polynomial<T>, zi: &ZeroIdeal<T>
) -> Polynomial<T> {
    // p^(2^i-1)
    let mut f = p.clone();
    for i in 0..zi.n {
        // p^(2^i)
        let g = compose(&f, p, zi).simplified(zi);
        if g.is_id() {
            // This will incorrectly say ord(X)=2, but whatever.
            crate::log(&format!("log(ord(p)) = {}", i + 1));
            return f;
        }

        f = compose(&f, &g, zi).simplified(zi);
    }

    assert!(false, "Failed to invert {}", p);
    Polynomial::zero()
}

/// Invert using Newton's method.
fn invert_newton<T: UniformNum>(
    p: &Polynomial<T>, zi: &ZeroIdeal<T>
) -> Polynomial<T> {
    // Initialize g with the initial guess Q(X)=X.
    let mut q = Polynomial::from_coeffs(&[T::zero(), T::one()]);

    let mut it = 0;

    // Do the Newton iterations.
    loop {
        assert!(it <= zi.n * 2, "Failed to compute the inverse\
                in a reasonable number of iterations.");

        // Compute the composition.
        let mut comp = compose(p, &q, zi).simplified(zi);

        // Do we already have p(q(x)) = x?
        if comp.is_id() {
            crate::log(&format!("Inverted in {} iterations", it));
            return q;
        }

        // Subtract X.
        // This is the quantity we want to make 0.
        comp.coeffs[1] -= T::one();

        // Update the guess.
        let qd = q.derivative();
        q -= &(&qd * &comp);
        q.simplify(zi);

        it += 1;
    }
}

/// Invert using interpolation.
fn invert_lagrange<T: UniformNum>(
    p: &Polynomial<T>, zi: &ZeroIdeal<T>
) -> Polynomial<T> {
    // Construct a system of linear congruences.
    let rows = zi.gen.last().unwrap().len();
    let cols = zi.gen.last().unwrap().len();

    // Construct the Vandermonde matrix.
    let mut a = Matrix::<T>::zero(rows, cols);
    let mut i = T::zero();
    for r in 0..rows {
        let mut j = T::one();
        let x = p.eval(i);
        for c in 0..cols {
            a[(r, c)] = j;
            j *= x;
        }

        i += T::one();
    }

    // Construct the vector of values of the polynomial.
    let mut b = Vector::<T>::zero(rows);
    let mut i = T::zero();
    for r in 0..rows {
        b[r] = i;
        i += T::one();
    }

    let l = congruence_solver::solve_congruences(a, &b);
    //crate::log(&format!("The kernel has dimension {}.", l.basis.len()));
    for b in &l.basis {
        let k = Polynomial::from_coeffs(b.entries());
        //assert!(k.simplified(zi).is_zero(),
        //    "Polynomial in the kernel is non-null.");
        if !k.clone().simplified(zi).is_zero() {
            crate::log(&format!("Polynomial in kernel is not null: {}", k));
        }
    }

    Polynomial::from_coeffs(l.offset.entries()).simplified(zi)
}

/// Computes the composition of two polynomials.
fn compose<T: UniformNum>(
    p: &Polynomial<T>,
    q: &Polynomial<T>,
    zi: &ZeroIdeal<T>
) -> Polynomial<T> {
    // We are using Horner's method to evaluate the polynomial `p` at `q(x)`.

    // Iterate over the coefficients in reverse order.
    let mut iter = p.coeffs.iter().rev();

    // The last coefficient is the initial value.
    let mut r = Polynomial::constant(iter.next().map_or(T::zero(), |c| *c));

    for c in iter {
        r *= q;
        r += *c;
        r.reduce(zi);
    }

    r
}

/// Used internally as a function to Iterator::fold.
fn parity<T: UniformNum>(acc: bool, i: &T) -> bool {
    match *i & T::one() != T::zero() {
        true => !acc,
        false => acc,
    }
}

/// Is this a permutation polynomial?
fn is_perm_poly<T: UniformNum>(f: &Polynomial<T>) -> bool {
    f.coeffs.get(1).map_or(false, |i| *i & T::one() != T::zero())
        && f.coeffs.iter().skip(2).step_by(2).fold(true, parity)
        && f.coeffs.iter().skip(3).step_by(2).fold(true, parity)
}

fn rand_poly_impl<T>() -> String
    where 
        T: UniformNum + std::fmt::Display,
        Standard: Distribution<T>,
{
    let mut rng = rand::thread_rng();
    let zi = ZeroIdeal::<T>::init();
    // This is the smallest degree possible that can represent any permutation
    // that has a polynomial representation.
    let degree = zi.gen.last().unwrap().len() - 1;

    // Create the polynomial. 
    let mut p = Polynomial {
        coeffs: vec![T::zero(); degree + 1]
    };

    // Initialize the coefficients with random values.
    for c in &mut p.coeffs {
        *c = rng.gen();
    }

    // a_1 has to be odd.
    if p.coeffs[1] & T::one() == T::zero() {
        p.coeffs[1] += T::one();
    }

    // a_2 + a_4 + ... has to be even.
    if p.coeffs.iter().skip(2).step_by(2).fold(false, parity) {
        let dist = Uniform::from(1..=degree/2);
        let i = dist.sample(&mut rng);
        p.coeffs[2*i] += T::one();
    }

    // a_3 + a_5 + ... has to be even.
    if p.coeffs.iter().skip(3).step_by(2).fold(false, parity) {
        let dist = Uniform::from(1..=(degree-1)/2);
        let i = dist.sample(&mut rng);
        p.coeffs[2*i+1] += T::one();
    }

    p.simplify(&zi);
    p.to_string()
}


/// Parse a polynomial.
/// Either as a space-separated list of coefficients a_d ... a_0,
/// or as a polynomial expression 4x^2 + 3x + 2.
fn parse_poly<T: NumAssign + Copy>(
    mut poly: String
) -> Result<Polynomial<T>, String> {
    if !poly.is_ascii() {
        return Err("Non-ascii input.".into());
    }

    poly.make_ascii_lowercase();

    // If the polynomial contains x then it is an actual expression.
    // Otherwise, it is a list of coefficients.
    let mut coeffs = Vec::with_capacity(8);
    if poly.contains('x') {
        poly.retain(|c| c != ' ');
        let p = poly.as_bytes();
        let mut i = 0;
        let mut last_i = usize::MAX;
        while i < p.len() {
            if i == last_i {
                return Err("Got stuck while parsing polynomial. This is a bug.".into());
            }
            last_i = i;

            // Parse the sign.
            let sign = match p[i] {
                b'+' => { i += 1; false },
                b'-' => { i += 1; true },
                _ => false,
            };

            // Parse the coefficient.
            let mut c = T::one();
            if p[i].is_ascii_digit() {
                let start = i;
                while i < p.len() && p[i].is_ascii_digit() {
                    i += 1;
                }

                c = <T as Num>::from_str_radix(&poly[start..i], 10)
                    .map_err(|_|
                        "Failed to parse coefficient.".to_owned()
                    )?;

                if i < p.len() && p[i] == b'*' {
                    i += 1;
                }
            }

            if sign {
                c = T::zero() - c;
            }

            // Parse the exponent.
            let mut e = 0;

            // Skip past the `x`.
            if i < p.len() && p[i] == b'x' {
                i += 1;
                e = 1;

                // If there is an exponent, parse it.
                if i < p.len() && p[i] == b'^' {
                    i += 1;
                    if !p[i].is_ascii_digit() {
                        return Err("Failed to parse exponent.".into());
                    }
                    e = 0;
                    while i < p.len() {
                        if !p[i].is_ascii_digit() {
                            break;
                        }

                        e *= 10;
                        e += (p[i] - b'0') as usize;
                        i += 1;
                    }
                }
            }

            if e >= coeffs.len() {
                coeffs.resize(e + 1, T::zero());
            }

            coeffs[e] = c;
        }
    } else {
        for c in poly.split_ascii_whitespace() {
            let c = <T as Num>::from_str_radix(c, 10)
                .map_err(|_| "Failed to parse coefficient.".to_owned())?;
            coeffs.push(c);
        }
        coeffs.reverse();
    }

    let p = Polynomial { coeffs };

    Ok(p.truncated())
}

/// The ideal of all polynomial expressions that evaluate to 0.
pub struct ZeroIdeal<T> {
    /// Mod 2^n.
    n: usize,

    /// The generators of the ideal.
    gen: Vec<Polynomial<T>>,
}

impl<T: UniformNum> ZeroIdeal<T> {
    pub fn init() -> Self {
        let n = std::mem::size_of::<T>() * 8;

        let mut gen = Vec::new();

        // div stores how often 2 divides i!.
        // It is successively updated.
        let mut div = 0usize;
        for i in (2usize..).step_by(2) {
            div += i.trailing_zeros() as usize;

            // If the exponent would be negative
            // then add the last generator and stop.
            if n <= div {
                let mut p = Polynomial::<T>::one();

                let mut j = T::zero();
                for _ in 0..i {
                    // Multiply the current polynomial by (x-j).
                    p.mul_lin(j);
                    j += T::one();
                }

                p.truncate();

                gen.push(p);
                break;
            }

            // Compute the exponent.
            let e = n - div;

            // Let's build the polynomial.
            let mut p = Polynomial::<T>::one();

            let mut j = T::zero();
            for _ in 0..i {
                // Multiply the current polynomial by (x-j).
                p.mul_lin(j);
                j += T::one();
            }

            p <<= e;
            p.truncate();

            gen.push(p);
        }

        Self { n, gen }
    }
}

impl<T: UniformNum> Polynomial<T> {
    /// Returns a simplified polynomial.
    pub fn simplified(mut self, zi: &ZeroIdeal<T>) -> Self {
        self.simplify(zi);
        self
    }

    /// Simplifies a polynomial by adding a polynomial in the zero ideal.
    pub fn simplify(&mut self, zi: &ZeroIdeal<T>) {
        let mut coeff = self.len() - 1;

        for gen in zi.gen.iter().rev() {
            let gen_len = gen.len() - 1;

            while coeff >= gen_len {
                let m = self.coeffs[coeff] / gen.coeffs[gen_len];
                if m != T::zero() {
                    let iter = (&mut self.coeffs[coeff-gen_len..=coeff])
                        .iter_mut().zip(gen.coeffs.iter());

                    for (p, g) in iter {
                        *p -= m * *g;
                    }
                }
                coeff -= 1;
            }
        }

        self.truncate();
    }

    /// Reduce the degree of the polynomial as much as possible
    /// using the generator of the highest degree.
    pub fn reduce(&mut self, zi: &ZeroIdeal<T>) {
        let gen = zi.gen.last().unwrap();
        let gen_len = gen.len() - 1;
        while self.len() >= gen.len() {
            let c = self.coeffs.pop().unwrap();
            for i in 0..gen_len {
                let j = self.len() - gen_len + i;
                self.coeffs[j] -= c * gen.coeffs[i];
            }
        }

        self.truncate();
    }
}