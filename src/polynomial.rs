use std::ops::{Add, AddAssign, Sub, Mul, MulAssign, SubAssign, ShlAssign};
use std::fmt::{self, Write, Display, Formatter};
use num_traits::{Num, NumAssign};

#[derive(Debug, Clone)]
pub struct Polynomial<T> {
    pub coeffs: Vec<T>,
}

impl<T> Polynomial<T> {
    pub fn zero() -> Self {
        Self { coeffs: Vec::new() }
    }

    pub fn constant(c: T) -> Self {
        Self { coeffs: vec![c] }
    }

    pub fn from_coeffs<U: Into<T> + Clone>(v: &[U]) -> Self {
        Self {
            coeffs: v.iter().map(|e| e.clone().into()).collect()
        }
    }

    /// The number of coefficients.
    pub fn len(&self) -> usize {
        self.coeffs.len()
    }

    //
    // Most functions below only work when the polynomial is truncated.
    //

    /// Returns the degree of the polynomial.
    /// The degree of 0 is defined to be -1.
    pub fn degree(&self) -> isize {
        self.coeffs.len() as isize - 1
    }

    /// Is this the zero polynomial?
    pub fn is_zero(&self) -> bool {
        self.len() == 0
    }
}

impl<T: NumAssign + Copy> Polynomial<T> {
    /// The constant 1 function.
    pub fn one() -> Self {
        Self::constant(T::one())
    }

    /// Is this the identity polynomial P(X)=X?
    pub fn is_id(&self) -> bool {
        self.coeffs == &[T::zero(), T::one()]
    }

    /// Removes leading zero coefficients.
    pub fn truncate(&mut self) {
        let num_zero = self.coeffs
            .iter()
            .rev()
            .take_while(|c| **c == T::zero())
            .count();

        self.coeffs.truncate(self.len() - num_zero);
    }

    /// Returns the truncated polynomial.
    pub fn truncated(mut self) -> Self {
        self.truncate();
        self
    }

    /// Evaluate the polynomial at x.
    pub fn eval(&self, x: T) -> T {
        // This is Horner's method.
        
        // Iterate over the coefficients in reverse order.
        let mut iter = self.coeffs.iter().rev().cloned();

        // The last coefficient is the initial value.
        let mut r = iter.next().unwrap_or(T::zero());

        for c in iter {
            r *= x;
            r += c;
        }

        r
    }

    /// Multiplies the polynomial by (X-a).
    pub fn mul_lin(&mut self, a: T) {
        // p(x) * (x-a) = p(x) * x - p(x) * a

        // Shift every coefficient to the left
        // which corresponds to a multiplication by x.
        self.coeffs.insert(0, T::zero());

        // Now subtract `a` times the original polynomial.
        for i in 0..self.coeffs.len() - 1 {
            let m = a * self.coeffs[i+1];
            self.coeffs[i] -= m;
        }
    }

    /// Computes the formal derivative of the polynomial.
    pub fn derivative(&self) -> Self {
        if self.len() == 0 {
            return Self::zero();
        }

        let mut coeffs = Vec::with_capacity(self.len() - 1);
        let mut d = T::one();
        for c in self.coeffs[1..].iter() {
            coeffs.push(*c * d);
            d += T::one();
        }

        Self { coeffs }
    }
}

impl<T: Num + Copy + Display> Polynomial<T> {
    pub fn to_tex(&self) -> String {
        let mut s = String::new();

        // Iterator over the non-zero coefficients.
        let mut iter = self.coeffs
            .iter()
            .enumerate()
            .rev()
            .filter(|(e, c)| **c != T::zero());

        let write_term = |s: &mut String, e, c| {
            if e == 0 {
                write!(s, "{}", c);
            } else {
                if c != T::one() {
                    write!(s, "{}", c);
                }

                write!(s, "X");

                if e != 1 {
                    write!(s, "^{{{}}}", e);
                }
            }
        };

        match iter.next() {
            None => { write!(s, "0"); },
            Some((e, c)) => write_term(&mut s, e, *c),
        };

        for (e, c) in iter {
            s.push_str("+");
            write_term(&mut s, e, *c);
        }

        s
    }
}

impl<T: Num + Display> Display for Polynomial<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Iterator over the non-zero coefficients.
        let mut iter = self.coeffs
            .iter()
            .enumerate()
            .rev()
            .filter(|(e, c)| **c != T::zero());

        match iter.next() {
            None => write!(f, "0")?,
            Some((e, c)) => if e == 0 {
                write!(f, "{}", c)?
            } else {
                write!(f, "{}x^{}", c, e)?
            },
        };

        for (e, c) in iter {
            if e == 0 {
                write!(f, " + {}", c)?;
            } else {
                write!(f, " + {}x^{}", c, e)?;
            }
        }

        Ok(())
    }
}

impl<T: Num + Copy> Add for &Polynomial<T> {
    type Output = Polynomial<T>;
    fn add(self, rhs: Self) -> Self::Output {
        // Order the polynomials by degree.
        let (min, max) = match self.len() >= rhs.len() {
            true => (rhs, self),
            false => (self, rhs),
        };

        let mut coeffs = Vec::with_capacity(max.len());

        // Add up all coefficients that exist in both.
        self.coeffs.iter()
            .zip(rhs.coeffs.iter())
            .for_each(|(l, r)| coeffs.push(*l + *r));

        // Push the remaining coefficients.
        for c in &max.coeffs[min.len()..] {
            coeffs.push(*c);
        }
        
        Polynomial { coeffs }
    }
}

impl<T: NumAssign + Copy> AddAssign<&Polynomial<T>> for Polynomial<T> {
    fn add_assign(&mut self, rhs: &Self) {
        // Add the coefficients that exist in both.
        self.coeffs.iter_mut()
            .zip(rhs.coeffs.iter())
            .for_each(|(l, r)| *l += *r);

        // Push the remaining coefficients should rhs have more.
        for c in &rhs.coeffs[self.len()..] {
            self.coeffs.push(*c);
        }
    }
}

impl<T: NumAssign + Copy> AddAssign<T> for Polynomial<T> {
    fn add_assign(&mut self, rhs: T) {
        if self.coeffs.is_empty() {
            self.coeffs.push(rhs);
        } else {
            self.coeffs[0] += rhs;
        }
    }
}

impl<T: Num + Copy> Sub for &Polynomial<T> {
    type Output = Polynomial<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        // Order the polynomials by degree.
        let (min, max) = match self.len() >= rhs.len() {
            true => (rhs, self),
            false => (self, rhs),
        };

        let mut coeffs = Vec::with_capacity(max.len());

        // Add up all coefficients that exist in both.
        self.coeffs.iter()
            .zip(rhs.coeffs.iter())
            .for_each(|(l, r)| coeffs.push(*l - *r));

        // Push the remaining coefficients.
        for c in &max.coeffs[min.len()..] {
            coeffs.push(*c);
        }
        
        Polynomial { coeffs }
    }
}

impl<T: NumAssign + Copy> SubAssign<&Polynomial<T>> for Polynomial<T> {
    fn sub_assign(&mut self, rhs: &Self) {
        // Add the coefficients that exist in both.
        self.coeffs.iter_mut()
            .zip(rhs.coeffs.iter())
            .for_each(|(l, r)| *l -= *r);

        // Push the remaining coefficients should rhs have more.
        for c in &rhs.coeffs[self.len()..] {
            self.coeffs.push(T::zero() - *c);
        }
    }
}

impl<T: NumAssign + Copy> Mul for &Polynomial<T> {
    type Output = Polynomial<T>;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut coeffs = vec![T::zero(); self.len() + rhs.len() - 1];
        for (i, c) in rhs.coeffs.iter().enumerate() {
            for (j, d) in self.coeffs.iter().enumerate() {
                coeffs[i + j] += *c * *d;
            }
        }

        Polynomial { coeffs }
    }
}

impl<T: NumAssign + Copy> MulAssign<&Polynomial<T>> for Polynomial<T> {
    fn mul_assign(&mut self, rhs: &Polynomial<T>) {
        *self = &*self * rhs;
    }
}

impl<T: ShlAssign<usize>> ShlAssign<usize> for Polynomial<T> {
    fn shl_assign(&mut self, m: usize) {
        for c in &mut self.coeffs {
            *c <<= m;
        }
    }
}