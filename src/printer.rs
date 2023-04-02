//! Prints the expressions in different formats.
//! The implementation is a bit disgusting.

use std::fmt::{self, Display, Write, Formatter};
use crate::uniform_expr::{UExpr, LUExpr};
use crate::pages::Width;
use crate::numbers::UniformNum;

use wasm_bindgen::prelude::*;
use num_traits::{Zero, One};

/// Determines how the result will be printed.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Printer {
    /// Some default.
    Default,

    /// Print a C function.
    C,

    /// Print a rust function.
    Rust,

    /// Tex expression.
    Tex,
}

impl Printer {
    /// Abbreviation to turn a UExpr into something
    /// that is `Display`ed correctly.
    fn u(self, e: &'_ UExpr) -> UExprPrinter<'_> {
        UExprPrinter { p: self, e }
    }

    pub fn print_uexpr(self, e: &UExpr) -> String {
        self.u(e).to_string()
    }

    pub fn print_luexpr<T: UniformNum>(self, e: &LUExpr<T>) -> String {
        let mut s = String::with_capacity(e.0.len() * 8);
        match self {
            Printer::C | Printer::Default => {
                let ty = match std::mem::size_of::<T>() {
                    1 => "uint8_t",
                    2 => "uint16_t",
                    4 => "uint32_t",
                    8 => "uint64_t",
                    16 => "uint128_t",
                    _ => unreachable!(),
                };

                let vars = e.vars();
                write!(&mut s, "{} f(", ty);
                for v in &vars {
                    write!(&mut s, "{} {}, ", ty, v);
                }

                // Remove the last ', '.
                s.pop();
                s.pop();
                s += ") {\n\treturn ";
                self.print_luexpr_impl(&mut s, e, "");
                s += ";\n}"
            },
            Printer::Rust => {
                let (ty, const_suffix) = match std::mem::size_of::<T>() {
                    1   => ("Wrapping<u8>", "u8"),
                    2   => ("Wrapping<u16>", "u16"),
                    4   => ("Wrapping<u32>", "u32"),
                    8   => ("Wrapping<u64>", "u64"),
                    16  => ("Wrapping<u128>", "u128"),
                    _ => unreachable!(),
                };

                let vars = e.vars();
                s += "fn f(";
                for v in &vars {
                    write!(&mut s, "{}: {},", v, ty);
                }

                // Remove the last ', '.
                s.pop();
                s.pop();
                s += ") -> ";
                s += ty;
                s += " {\n\t";
                self.print_luexpr_impl(&mut s, e, const_suffix);
                s += "\n}"
            },
            Printer::Tex => self.print_luexpr_impl(&mut s, e, ""),
        }

        return s;
    }

    fn print_luexpr_impl<T: UniformNum>(
        self,
        s: &mut String,
        e: &LUExpr<T>,
        const_suffix: &str,
    ) {
        let mut iter = e.0.iter()
            .filter(|(i, _)| *i != T::zero())
            .map(|(i, e)| (*i, e));

        let (mut i, e) = match iter.next() {
            Some(t) => t,
            None => return *s += "0",
        };

        let fmt_term = |s: &mut String, mut i: T, e: &UExpr| {
            let unary = e.is_unary();
            let e = self.u(e);
            if i == T::one() {
                if unary {
                    write!(s, "{}", e);
                } else {
                    write!(s, "({})", e);
                }
            } else {
                if self == Self::Rust {
                    if unary {
                        write!(s, "Wrapping({}{})*{}", i, const_suffix, e);
                    } else {
                        write!(s, "Wrapping({}{})*({})", i, const_suffix, e);
                    }
                    return;
                }

                let op = match self {
                    Self::Default | Self::C | Self::Rust => "*",
                    Self::Tex => "\\cdot ",
                };

                if unary {
                    write!(s, "{}{}{}", i, op, e);
                } else {
                    write!(s, "{}{}({})", i, op, e);
                }
            }
        };

        if i.print_negative() {
            write!(s, "-");
            i = T::zero() - i
        }
        fmt_term(s, i, e);

        for (i, e) in iter {
            let j = if i.print_negative() {
                write!(s, " - ");
                T::zero() - i
            } else {
                write!(s, " + ");
                i
            };
            fmt_term(s, j, e);
        }
    }
}

struct UExprPrinter<'a> {
    p: Printer,
    e: &'a UExpr,
}

impl<'a> UExprPrinter<'a> {
    fn u<'b>(&self, e: &'b UExpr) -> UExprPrinter<'b> {
        UExprPrinter { p: self.p, e }
    }

    fn write_safe(
        &self, l: &UExpr, r: &UExpr, op: &str, f: &mut Formatter<'_>
    ) -> fmt::Result {
        let l = self.u(l);
        let r = self.u(r);
        if l.e.is_unary() {
            write!(f, "{} {}", l, op)?;
        } else {
            write!(f, "({}) {}", l, op)?;
        }

        if r.e.is_unary() {
            write!(f, " {}", r)
        } else {
            write!(f, " ({})", r)
        }
    }
}

impl<'a> Display for UExprPrinter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use UExpr::*;
        use Printer::*;
        match self.e {
            Ones => f.write_str("-1"),
            Var(c) => f.write_str(c),
            Not(i) => {
                let i = self.u(i);
                match self.p {
                    Default | C if i.e.is_unary() => write!(f, "~{}", i),
                    Default | C => write!(f, "~({})", i),
                    Rust if i.e.is_unary() => write!(f, "!{}", i),
                    Rust => write!(f, "!({})", i),
                    Tex => write!(f, "\\overline{{{}}}", i),
                }
            },
            And(l, r) => {
                match self.p {
                    Default | C | Rust => self.write_safe(l, r, "&", f),
                    Tex => self.write_safe(l, r, "\\land", f),
                }
            },
            Or(l, r) => {
                match self.p {
                    Default | C | Rust => self.write_safe(l, r, "|", f),
                    Tex => self.write_safe(l, r, "\\lor", f),
                }
            },
            Xor(l, r) => {
                match self.p {
                    Default | C | Rust => self.write_safe(l, r, "^", f),
                    Tex => self.write_safe(l, r, "\\oplus", f),
                }
            }
        }
    }
}