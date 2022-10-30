use wasm_bindgen::prelude::*;
use std::fmt::Display;
use std::num::Wrapping;
use crate::vector::Vector;
use crate::matrix::Matrix;
use super::{Bitness, bold, underbrace};
use crate::congruence_solver::{
    AffineLattice, ModN, diagonalize, solve_scalar_congruence
};

/// Stores the intermediate results during the computation of the solution.
#[wasm_bindgen]
pub struct SolveTrace {
    /// The diagonalization of the matrix.
    diag: String,

    /// The resulting diagonal system.
    scalar_system: String,

    /// Linear congruences and solutions.
    linear_solutions: String,

    /// Vector form of the solution.
    vector_solution: String,

    /// The final solution.
    final_solution: String,
}

#[wasm_bindgen]
impl SolveTrace {
    #[wasm_bindgen(getter)]
    pub fn diag(&self) -> String {
        self.diag.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn scalar_system(&self) -> String {
        self.scalar_system.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn linear_solutions(&self) -> String {
        self.linear_solutions.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn vector_solution(&self) -> String {
        self.vector_solution.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn final_solution(&self) -> String {
        self.final_solution.clone()
    }
}

fn solve_congruences_impl<T: ModN + Display>(
    a: Matrix<&str>, b: Vector<&str>
) -> Result<SolveTrace, String> {
    let a = a.try_map(|&e| T::from_str_radix(e, 10))
        .map_err(|(r, c, _)| format!("Failed to parse entry ({}, {}).", r+1, c+1))?;

    let b = b.try_map(|&e| T::from_str_radix(e, 10))
        .map_err(|(r, _)| format!("Failed to parse entry ({}, {}).", r+1, a.cols+1))?;
    
    let mut d = a.clone();
    let (s, t) = diagonalize(&mut d);
    assert!(&(&s * &a) * &t == d);

    let diag = format!("{}={}{}{}",
        underbrace(d.to_tex(), bold("D")),
        underbrace(s.to_tex(), bold("S")),
        underbrace(a.to_tex(), bold("A")),
        underbrace(t.to_tex(), bold("T"))
    );

    // Transform the vector b.
    // We could already do this in diagonalize if we really wanted.
    let b_new = (&s * &b);

    let scalar_system = format!("{}\\mathbf{{x'}}={}{}={}",
        underbrace(d.to_tex(), bold("D")),
        underbrace(s.to_tex(), bold("S")),
        underbrace(b.to_tex(), bold("b")),
        b_new.to_tex()
    );

    let b = b_new;

    let min_dim = d.min_dim();

    if let Some(i) = b.iter()
        .skip(min_dim)
        .position(|e| *e != T::zero()) {
        let i = min_dim + i;
        let linear_solutions = format!(
            "\\text{{Row {}: }} 0={}\\implies \\text{{No solution}}",
            i + 1, b[i]
        );
        return Ok(SolveTrace {
            diag,
            scalar_system,
            linear_solutions,
            vector_solution: String::new(),
            final_solution: String::new()
        });
    }

    // Some solution to the system.
    let mut offset = Vector::zero(d.cols);

    // The basis of the kernel.
    let mut basis = Vec::new();

    let mut linear_solutions = "\\begin{align}".to_owned();
    
    // Variable index for basis.
    let mut j = 1;

    // Solve the scalar linear congruences.
    for i in 0..d.min_dim() {
        linear_solutions += &format!(
            "{}x'_{{{}}}&={} &\\implies ", d[(i, i)], i + 1, b[i]);
        let (x, kern) = match solve_scalar_congruence(d[(i, i)], b[i]) {
            // If there is no solution,
            // then the whole system does not have a solution.
            None => {
                linear_solutions += "\\text{No solution!}&\\end{align}";
                return Ok(SolveTrace {
                    diag,
                    scalar_system,
                    linear_solutions,
                    vector_solution: String::new(),
                    final_solution: String::new(),
                });
            },
            Some(s) => s,
        };

        if kern == T::zero() {
            linear_solutions += &format!("x'_{{{}}}&={}\\\\", i + 1, x);
        } else {
            linear_solutions += &format!(
                "x'_{{{}}}&={}+{}a_{{{}}}\\\\", i + 1, x, kern, j
            );
            j += 1;
        }

        // The particular solution is an entry is
        // the particular solution of the whole system.
        offset[i] = x;

        // If the kernel is zero, then the vector is zero for sure.
        if kern != T::zero() {
            let mut v = Vector::zero(d.cols);
            v[i] = kern;
            basis.push(v);
        }
    }

    // If there are more variables then equations
    // then there are no restrictions on the variables
    // from index d.rows
    for i in d.rows..d.cols {
        let mut v = Vector::zero(d.cols);
        v[i] = T::one();
        basis.push(v);

        linear_solutions += &format!(
            "&&x'_{{{}}}&=a_{{{}}}\\\\", i + 1, j
        );
        j += 1;
    }

    linear_solutions += "\\end{align}";

    let mut solution = AffineLattice {
        offset,
        basis,
    };

    let x_old = solution.to_tex_brace();
    let vector_solution = format!("\\mathbf{{x'}}={}", x_old);

    solution.offset = (&t * &solution.offset);
    for v in &mut solution.basis {
        *v = (&t * &*v);
    }

    let final_solution = format!("\\mathbf{{x}}={}{}={}",
        underbrace(t.to_tex(), bold("T")),
        underbrace(x_old, bold("x'")),
        solution.to_tex()
    );

    Ok(SolveTrace {
        diag,
        scalar_system,
        linear_solutions,
        vector_solution,
        final_solution,
    })
}

#[wasm_bindgen]
pub fn solve_congruences(matrix_str: String, bit: Bitness) -> Result<SolveTrace, String> {
    // The number of rows is the number of lines.
    let rows = matrix_str.lines().count();
    if rows == 0 {
        return Err("Empty matrix.".into());
    }

    // Get the number of columns from the first line.
    let cols = matrix_str.lines()
        .next()
        .unwrap()
        .split_ascii_whitespace()
        .count() - 1;
    if cols == 0 {
        return Err("Empty matrix.".into());
    }

    let mut a = Matrix::<&str>::uniform(rows, cols, "");
    let mut b = Vector::<&str>::uniform(rows, "");

    for (i, l) in matrix_str.lines().enumerate() {
        // Is the number of elements in this row the same as in the first.
        let mut ok = false;
        for (j, e) in l.split_ascii_whitespace().enumerate() {
            if (j < cols) {
                a[(i, j)] = e;
            } else if (j == cols) {
                b[i] = e;
                ok = true;
            } else {
                ok = false;
                break;
            }
        }

        if !ok {
            return Err(std::format!("Row {} has a different number of \
                entries than the first row.", i + 1));
        }
    }

    match bit {
        Bitness::U8 => solve_congruences_impl::<Wrapping<u8>>(a, b),
        Bitness::U16 => solve_congruences_impl::<Wrapping<u16>>(a, b),
        Bitness::U32 => solve_congruences_impl::<Wrapping<u32>>(a, b),
        Bitness::U64 => solve_congruences_impl::<Wrapping<u64>>(a, b),
        Bitness::U128 => solve_congruences_impl::<Wrapping<u128>>(a, b),
    }
}