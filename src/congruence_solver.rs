use std::fmt::Display;
use crate::matrix::Matrix;
use crate::vector::Vector;
use crate::numbers::UnsignedInt;

pub struct AffineLattice<T> {
    pub offset: Vector<T>,
    pub basis: Vec<Vector<T>>,
}

impl<T> AffineLattice<T> {
    pub fn empty() -> Self {
        Self {
            offset: Vector::empty(),
            basis: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.offset.is_empty()
    }
}

impl<T: UnsignedInt> AffineLattice<T> {
    pub fn to_tex(&self) -> String {
        let mut s = self.offset.to_tex();
        for (i, b) in self.basis.iter().enumerate() {
            s += &std::format!("+a_{}{}", i + 1, b.to_tex());
        }
        s
    }

    pub fn to_tex_brace(&self) -> String {
        if self.basis.is_empty() {
            self.to_tex()
        } else {
            format!("\\left({}\\right)", self.to_tex())
        }
    }
}


/// Solves ax=b mod n where n = 2^8 for u8.
/// Returns None if there is no solution.
/// Otherwise returns all solutions in the form (c, d)
/// where c+di are all solutions.
pub fn solve_scalar_congruence<T: UnsignedInt>(
    a: T, b: T
) -> Option<(T, T)> {
    // Handle the case that a is zero, so we don't have to think about it.
    if a == T::zero() {
        return (b == T::zero()).then_some((T::zero(), T::one()));
    }

    // We are basically going to use the extended euclidean algorithm on
    // the diophantine equation ax+ny=b where n is the number of values
    // (2^8 for u8).
    // But n doesn't fit into T, so we have to do a hack in the first step.
    // Usually we'd divide n by a but instead we divide n-a by a and add 1.
    // This makes the code structurally uglier, but otherwise I'm pretty
    // much just following the pseudo code on wikipedia.
    
    let (mut old_r, mut r) = (T::zero(), a);
    let (mut old_t, mut t) = (T::zero(), T::one());
    let mut q = (T::zero() - a) / a + T::one();

    loop {
        (old_r, r) = (r, old_r - q * r);
        (old_t, t) = (t, old_t - q * t);
        if r == T::zero() {
            break;
        }
        q = old_r / r;
    }

    // old_r is gcd(a, n).
    let gcd = old_r;

    // There is a solution iff gcd divides b, but we can also just check ax=b.
    // old_t is the Bezout coefficient: a*old_t=gcd(a, n) mod n.
    let x = b / gcd * old_t;
    if a * x != b {
        return None;
    }

    // The kernel is n / gcd which happens to be in t.
    // If the kernel is greater than n/2 we can take -t
    // which is smaller.
    let kern = std::cmp::min(t, T::zero() - t);

    Some((x, t))
}

/// Solves a system of linear congruences Ax=b.
pub fn solve_congruences<T: UnsignedInt>(
    mut a: Matrix<T>, b: &Vector<T>
) -> AffineLattice<T> {
    debug_assert!(a.rows == b.dim, "Invalid system of congruences");

    // Diagonalize the system.
    let (s, t) = diagonalize(&mut a);
    
    // Transform the vector b.
    // We could already do this in diagonalize if we really wanted.
    let b = (&s * b);

    // If there is a non-zero entry in b at index >a.min_dim()
    // then the system has no solution, since the corresponding
    // row in a is zero, so we are solving 0=x.
    if b.iter().skip(a.min_dim()).any(|e| *e != T::zero()) {
        return AffineLattice::empty();
    }

    // Some solution to the system.
    let mut offset = Vector::zero(a.cols);

    // The basis of the kernel.
    let mut basis = Vec::new();

    // Solve the scalar linear congruences.
    for i in 0..a.min_dim() {
        let (x, kern) = match solve_scalar_congruence(a[(i, i)], b[i]) {
            // If there is no solution,
            // then the whole system does not have a solution.
            None => return AffineLattice::empty(),
            Some(s) => s,
        };

        // The particular solution is an entry is
        // the particular solution of the whole system.
        offset[i] = x;

        // If the kernel is zero, then the vector is zero for sure.
        if kern != T::zero() {
            let mut v = Vector::zero(a.cols);
            v[i] = kern;
            basis.push(v);
        }
    }

    // If there are more variables then equations
    // then there are no restrictions on the variables
    // from index d.rows
    for i in a.rows..a.cols {
        let mut v = Vector::zero(a.cols);
        v[i] = T::one();
        basis.push(v);
    }

    offset = (&t * &offset);
    for v in &mut basis {
        *v = (&t * &*v);
    }

    AffineLattice {
        offset,
        basis
    }
}

/// Computes a diagonal matrix D in-place
/// and returns matrices (S, T), such that D=SAT.
pub fn diagonalize<T: UnsignedInt>(
    a: &mut Matrix<T>
) -> (Matrix<T>, Matrix<T>) {
    // The matrices S and T are initialized to the identity.
    // S/T keeps track of the row/column operations.
    let mut s = Matrix::<T>::id(a.rows);
    let mut t = Matrix::<T>::id(a.cols);

    for i in 0..a.min_dim() {
        //
        // Eliminate row i and column i.
        //
        loop {
            // Is there a non-zero element in the column?
            let col_zero = a.col(i)
                .skip(i+1)
                .all(|e| *e == T::zero());

            if (!col_zero) {
                //
                // Eliminate the column.
                //

                // Find a pivot in the column.
                let pivot = a.col(i)
                    .enumerate()
                    .skip(i)
                    .filter(|e| *e.1 != T::zero())
                    .min_by_key(|e| e.1)
                    .map(|e| e.0)
                    .unwrap(); // We know there is a non-zero element.

                // Move the pivot to the beginning.
                a.swap_rows(i, pivot);
                s.swap_rows(i, pivot);

                // Try to eliminate every other entry in the column.
                for k in i+1..a.rows {
                    if a[(k, i)] != T::zero() {
                        let m = T::zero() - (a[(k, i)] / a[(i, i)]);
                        a.row_multiply_add(i, k, m);
                        s.row_multiply_add(i, k, m);
                    }
                }

                // Keep eliminating the column.
                continue;
            }

            // If we get here, the column is zero.

            // Is there a non-zero element in the row?
            let row_zero = a.row(i)
                .iter()
                .skip(i+1)
                .all(|e| *e == T::zero());
            
            // If the row is zero, then continue with the next row/column.
            if row_zero {
                break;
            }
            
            //
            // Eliminate the row.
            //

            // Find a pivot in the row.
            let pivot = a.row(i)
                .iter()
                .enumerate()
                .skip(i)
                .filter(|e| *e.1 != T::zero())
                .min_by_key(|e| e.1)
                .map(|e| e.0)
                .unwrap(); // We know there is a non-zero element.

            // Move the pivot to the beginning.
            a.swap_columns(i, pivot);
            t.swap_columns(i, pivot);

            // Try to eliminate every other entry in the row.
            for k in i+1..a.cols {
                if a[(i, k)] != T::zero() {
                    let m = T::zero() - (a[(i, k)] / a[(i, i)]);
                    a.col_multiply_add(i, k, m);
                    t.col_multiply_add(i, k, m);
                }
            }
        }
    }

    return (s, t);
}

/// Solves ax=b mod n.
/// Returns None if there is no solution.
/// Otherwise returns all solutions in the form (c, d)
/// where c+di are all solutions.
pub fn solve_scalar_congruence_mod<T: UnsignedInt>(
    a: T, b: T, n: T
) -> Option<(T, T)> {
    assert!(!n.is_zero());
    // Handle the case that a is zero, so we don't have to think about it.
    if a == T::zero() {
        return (b == T::zero()).then_some((T::zero(), T::one()));
    }

    let (mut old_r, mut r) = (a, n);
    let (mut old_t, mut t) = (T::zero(), T::one());
    while !r.is_zero() {
        let q = old_r / r;
        (old_r, r) = (r, old_r - q * r);
        (old_t, t) = (t, old_t - q * t);
    }

    // old_r is gcd(a, n).
    let gcd = old_r;

    // There is a solution iff gcd divides b, but we can also just check ax=b.
    // old_t is the Bezout coefficient: a*old_t=gcd(a, n) mod n.
    let x = b / gcd * old_t;
    if a * x != b {
        return None;
    }

    // The kernel is n / gcd which happens to be in t.
    // If the kernel is greater than n/2 we can take -t
    // which is smaller.
    let kern = std::cmp::min(t, T::zero() - t);

    Some((x, t))
}