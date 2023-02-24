use std::collections::BTreeSet;
use std::fmt::{self, Formatter, Display};
use std::ops::{Index, IndexMut};
use std::rc::Rc;
use num_traits::Num;

use crate::expr::Expr;
use crate::numbers::{UnsignedInt, UniformNum, int_from_it};

/// LUExpr is short for "Linear combination of Uniform Expressions"
/// These are the expressions for which rewrite rules can be efficiently
/// generated.
#[derive(Clone, Debug)]
pub struct LUExpr<T>(pub Vec<(T, UExpr)>);

impl<T: UniformNum> LUExpr<T> {
    /// Creates an expression that equals a constant.
    pub fn constant(c: T) -> Self {
        Self(vec![(T::zero() - c, UExpr::Ones)])
    }

    /// Create a uniform expression.
    pub fn from_uexpr(u: UExpr) -> Self {
        Self(vec![(T::one(), u)])
    }

    /// Creates an expression that equals a variable.
    pub fn var(name: String) -> Self {
        Self(vec![(T::one(), UExpr::Var(name))])
    }

    /// Returns all variables in the expression.
    /// This will include duplicates.
    pub fn vars(&self) -> Vec<String> {
        let mut v = BTreeSet::new();
        self.vars_impl(&mut v);
        v.into_iter().collect()
    }

    pub(crate) fn vars_impl(&self, v: &mut BTreeSet<String>) {
        for (_, e) in &self.0 {
            e.vars_impl(v);
        }
    }

    /// Evaluate an expression with a valuation for the occurring variables.
    pub fn eval(&self, v: &Valuation<T>) -> T {
        self.0.iter()
            .map(|(i, e)| *i * e.eval(v))
            .fold(T::zero(), T::add)
    }

    /// Parse a string to an expression.
    /// Note that this function is extremely limited
    /// and expects very specific syntax.
    /// It is used for convenience when testing things and
    /// not really meant to be used by something outside this crate.
    pub(crate) fn from_string(s: String) -> Option<Self> {
        let mut s = s.to_string();
        s.retain(|c| !c.is_whitespace());
        let mut it = s.chars().peekable();

        // This stores the current linear combination.
        let mut v = Vec::new();

        let mut neg = false;

        // Loop over the string/the summands.
        loop {
            // Are there still characters left?
            // If not then we're done.
            let mut c = match it.peek() {
                None => return Some(Self(v)),
                Some(c) => *c,
            };

            if c == '-' {
                neg = true;
                it.next();
                c = *it.peek()?;
            }

            // If this is a digit then we expect num*UExpr.
            if c.is_ascii_digit() {
                // Parse the number.
                let mut num = int_from_it(&mut it)?;

                // If the number is negative then negate it.
                if neg {
                    num = T::zero() - num;
                }

                // Is it the expected '*'?
                match it.peek() {
                    Some('*') => {
                        it.next();

                        // Parse the UExpr.
                        let e = UExpr::parse(&mut it, 0)?;

                        // Push it.
                        v.push((num, e));
                    },

                    // If this is a different character then we push -num*(-1).
                    _ => v.push((T::zero() - num, UExpr::Ones)),
                }
            } else {
                // We don't have a factor so just parse the UExpr.
                let e = UExpr::parse(&mut it, 0)?;

                let sign = match neg {
                    false => T::one(),
                    true => T::zero() - T::one(),
                };

                // Push sign*e.
                v.push((sign, e));
            }

            // If the next character is not a plus then we are done.
            match it.peek() {
                // Next part of the linear combination.
                Some('+') => { neg = false; it.next() }, // Skip the +.
                Some('-') => { neg = true; it.next() },

                // We consumed the whole input so we're good.
                None => return Some(Self(v)),

                // There is something left but we can't parse it.
                _ => return None,
            };
        }
    }

    pub fn to_expr(&self) -> Expr<T> {
        let mut it = self.0.iter()
            .filter(|(f, u)| !f.is_zero());

        // If the linear combination is empty,
        // then it always evaluates to 0.
        let Some((f, u)) = it.next() else {
            return Expr::zero()
        };

        // Lambda to convert the `coefficient * uexpr` into an expr.
        let term = |f: T, u: &UExpr| {
            if f.is_one() {
                u.to_expr()
            } else {
                Expr::Mul(Rc::new(Expr::Const(f)), Rc::new(u.to_expr()))
            }
        };

        // Iterate over the linear combination and update e.
        let mut e = term(*f, u);
        for (f, u) in it {
            e = Expr::Add(Rc::new(e), Rc::new(term(*f, u)));
        }

        e
    }
}

impl<T: UniformNum> From<UExpr> for LUExpr<T> {
    fn from(e: UExpr) -> Self {
        Self(vec![(T::one(), e)])
    }
}

impl<T: UniformNum> Display for LUExpr<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter()
            .filter(|(i, _)| *i != T::zero())
            .map(|(i, e)| (*i, e));
        let (mut i, e) = match iter.next() {
            Some(t) => t,
            None => return write!(f, "0"),
        };

        let fmt_term = |f: &mut Formatter<'_>, i: T, e: &UExpr| {
            if i == T::one() {
                write!(f, "{}", e)
            } else {
                if e.is_unary() {
                    write!(f, "{}*{}", i, e)
                } else {
                    write!(f, "{}*({})", i, e)
                }
            }
        };

        if i.print_negative() {
            write!(f, "-")?;
            i = T::zero() - i
        }
        fmt_term(f, i, e)?;

        for (i, e) in iter {
            let j = if i.print_negative() {
                write!(f, " - ")?;
                T::zero() - i
            } else {
                write!(f, " + ")?;
                i
            };
            fmt_term(f, j, e);
        }

        Ok(())
    }
}

/// Represents an expression that is uniform on all bits.
/// Note that the variant 'Ones' does not equal 1, but a 1 in every bit,
/// which is -1 in two's complement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UExpr {
    Ones,
    Var(String),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Xor(Box<Self>, Box<Self>),
}

impl UExpr {
    pub fn var(c: String) -> Self {
        Self::Var(c)
    }

    pub fn not(e: Self) -> Self {
        Self::Not(e.into())
    }

    pub fn and(l: Self, r: Self) -> Self {
        Self::And(l.into(), r.into())
    }

    pub fn or(l: Self, r: Self) -> Self {
        Self::Or(l.into(), r.into())
    }

    pub fn xor(l: Self, r: Self) -> Self {
        Self::Xor(l.into(), r.into())
    }

    /// Is the top-most operator unary.
    pub fn is_unary(&self) -> bool {
        use UExpr::*;
        match self {
            Ones | Var(_) | Not(_) => true,
            _ => false,
        }
    }

    /// Returns all variables in the expression.
    /// This will include duplicates.
    pub fn vars(&self) -> Vec<String> {
        let mut v = BTreeSet::new();
        self.vars_impl(&mut v);
        v.into_iter().collect()
    }

    // Should really be `pub(self)`.
    pub(crate) fn vars_impl(&self, v: &mut BTreeSet<String>) {
        use UExpr::*;
        match self {
            Ones            => {},
            Var(c)          => if !v.contains(c) { v.insert(c.clone()); },
            Not(e)          => e.vars_impl(v),
            And(e1, e2)     => { e1.vars_impl(v); e2.vars_impl(v) },
            Or(e1, e2)      => { e1.vars_impl(v); e2.vars_impl(v) },
            Xor(e1, e2)     => { e1.vars_impl(v); e2.vars_impl(v) },
        }
    }

    /// Evaluate an expression with a valuation for the occurring variables.
    pub fn eval<T: UniformNum>(&self, v: &Valuation<T>) -> T {
        use UExpr::*;
        match self {
            Ones            => T::zero() - T::one(), // -1
            Var(c)          => v[c],
            Not(e)          => !e.eval(v),
            And(e1, e2)     => e1.eval(v) & e2.eval(v),
            Or(e1, e2)      => e1.eval(v) | e2.eval(v),
            Xor(e1, e2)     => e1.eval(v) ^ e2.eval(v),
        }
    }

    /// Rename a variable.
    pub fn rename_var(&mut self, old: &str, new: &str) {
        use UExpr::*;
        match self {
            Ones        => (),
            Var(v)      => if v == old { v.clear(); v.push_str(new) },
            Not(e)      => e.rename_var(old, new),
            And(l, r)   => { l.rename_var(old, new); r.rename_var(old, new) },
            Or(l, r)    => { l.rename_var(old, new); r.rename_var(old, new) },
            Xor(l, r)   => { l.rename_var(old, new); r.rename_var(old, new) },
        }
    }

    pub(crate) fn write_safe(
        e1: &Self, e2: &Self, op: &str, f: &mut std::fmt::Formatter<'_>
    ) -> std::fmt::Result {
        if e1.is_unary() {
            write!(f, "{} {}", e1, op)?;
        } else {
            write!(f, "({}) {}", e1, op)?;
        }

        if e2.is_unary() {
            write!(f, " {}", e2)
        } else {
            write!(f, " ({})", e2)
        }
    }

    /// Parse a string to an expression.
    pub(crate) fn from_string<T: ToString>(s: T) -> Option<Self> {
        let mut s = s.to_string();
        s.retain(|c| !c.is_whitespace());
        let mut it = s.chars().peekable();

        Self::parse(&mut it, 0)
            .filter(|_| it.next().is_none())
    }

    pub(self) fn parse(
        it: &mut std::iter::Peekable<std::str::Chars>,
        pre: usize
    ) -> Option<Self> {
        use UExpr::*;

        let c = *it.peek()?;

        let mut e = if c == '(' {
            it.next();
            let e = Self::parse(it, 0)?;
            match it.next() {
                Some(')') => e,
                _ => return None,
            }
        } else if c == '~' || c == '!' {
            it.next();
            let e = Self::parse(it, 15)?;
            Not(Box::new(e))
        } else if c.is_alphabetic() {
            it.next();
            let mut var = String::from(c);
            loop {
                let Some(c) = it.peek() else {
                    break
                };

                if !c.is_alphanumeric() {
                    break
                }

                var.push(*c);
                it.next();
            }
            
            Var(var)
        } else if c == '-' {
            it.next();
            // Parse a -1.
            match it.next() {
                Some('1') => Ones,
                _ => return None,
            }
        } else {
            return None;
        };

        loop {
            let c = match it.peek() {
                None => return Some(e),
                Some(c) => *c,
            };

            let op_pre = match c {
                '|' => 1,
                '^' => 2,
                '&' => 3,
                _ => return Some(e),
            };

            if op_pre <= pre {
                return Some(e);
            }

            // If the current operators precedence is higher than
            // the one whose subexpressions we are currently parsing
            // then we need to finish this operator first.
            it.next();
            let rhs = Box::new(Self::parse(it, op_pre)?);
            let lhs = Box::new(e);
            e = match c {
                '&' => And(lhs, rhs),
                '|' => Or(lhs, rhs),
                '^' => Xor(lhs, rhs),
                _ => return None,
            };
        }
    }

    pub fn to_expr<T: UniformNum>(&self) -> Expr<T> {
        match self {
            UExpr::Ones => Expr::Const(T::zero() - T::one()),
            UExpr::Var(v) => Expr::Var(v.clone()),
            UExpr::Not(e) => Expr::Not(Rc::new(e.to_expr::<T>())),
            UExpr::And(l, r) => Expr::And(Rc::new(l.to_expr::<T>()), Rc::new(r.to_expr::<T>())),
            UExpr::Or(l, r) => Expr::Or(Rc::new(l.to_expr::<T>()), Rc::new(r.to_expr::<T>())),
            UExpr::Xor(l, r) => Expr::Xor(Rc::new(l.to_expr::<T>()), Rc::new(r.to_expr::<T>())),
        }
    }
}

impl std::fmt::Display for UExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UExpr::*;
        match self {
            Ones => write!(f, "-1"),
            Var(c) => write!(f, "{}", c),
            And(e1, e2)   => Self::write_safe(e1, e2, "&", f),
            Or(e1, e2)    => Self::write_safe(e1, e2, "|", f),
            Xor(e1, e2)   => Self::write_safe(e1, e2, "^", f),
            Not(e) =>
                if e.is_unary() {
                    write!(f, "~{}", e)
                } else {
                    write!(f, "~({})", e)
                },
        }
    }
}

/// Stores values that should be substituted into variables.
#[derive(Debug)]
pub struct Valuation<T> {
    /// The key value pairs are stored as a Vector
    /// because I doubt a hashmap/tree would be faster
    /// when there are so few variables.
    vals: Vec<(String, T)>,
}

impl<T: Num> Valuation<T> {
    /// Initializes a valuation from a list of variables
    /// each of which will be Initialized to 0.
    pub fn zero(vars: Vec<String>) -> Self {
        let vals = vars.into_iter()
            .map(|c| (c, T::zero()))
            .collect();

        Self { vals }
    }
}

impl<T> Index<&str> for Valuation<T> {
    type Output = T;
    fn index(&self, index: &str) -> &Self::Output {
        &self.vals.iter()
            .find(|(name, _)| *name == index)
            .unwrap().1
    }
}

impl<T> IndexMut<&str> for Valuation<T> {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        &mut self.vals.iter_mut()
            .find(|(name, _)| *name == index)
            .unwrap().1
    }
}