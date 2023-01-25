//! Prints the expressions in different formats.
//! The implementation is a bit disgusting.

use std::fmt::{self, Display, Write, Formatter};
use crate::uniform_expr::{UExpr, LUExpr};
use crate::pages::Bitness;
use crate::numbers::UniformNum;

use wasm_bindgen::prelude::*;
use num_traits::{Zero, One};

/// Determines how the result will be printed.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Printer {
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
    fn u(self, e: &'_ UExpr) -> UExprHelper<'_> {
        UExprHelper { p: self, e }
    }

    pub fn print_uexpr(self, e: &UExpr) -> String {
        self.u(e).to_string()
    }

    pub fn print_luexpr<T: UniformNum>(self, e: &LUExpr<T>, bits: Bitness) -> String {
        let mut s = String::with_capacity(e.0.len() * 8);
        match self {
            Printer::C => {
                let ty = match bits {
                    Bitness::U8 => "uint8_t",
                    Bitness::U16 => "uint16_t",
                    Bitness::U32 => "uint32_t",
                    Bitness::U64 => "uint64_t",
                    Bitness::U128 => "uint128_t",
                };

                let mut vars = Vec::new();
                e.vars(&mut vars);
                vars.sort();
                vars.dedup();
                s += ty;
                s += " f(";
                for v in vars {
                    s += ty;
                    s += " ";
                    s.push(v);
                    s += ", ";
                }

                // Remove the last ', '.
                s.pop();
                s.pop();
                s += ") {\n\treturn ";
                self.print_luexpr_impl(&mut s, e, "");
                s += ";\n}"
            },
            Printer::Rust => {
                let (ty, const_suffix) = match bits {
                    Bitness::U8     => ("Wrapping<u8>", "u8"),
                    Bitness::U16    => ("Wrapping<u16>", "u16"),
                    Bitness::U32    => ("Wrapping<u32>", "u32"),
                    Bitness::U64    => ("Wrapping<u64>", "u64"),
                    Bitness::U128   => ("Wrapping<u128>", "u128"),
                };

                let mut vars = Vec::new();
                e.vars(&mut vars);
                vars.sort();
                vars.dedup();
                s += "fn f(";
                for v in vars {
                    s.push(v);
                    s += ": ";
                    s += ty;
                    s += ", ";
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
                    Self::C | Self::Rust => "*",
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

struct UExprHelper<'a> {
    p: Printer,
    e: &'a UExpr,
}

impl<'a> UExprHelper<'a> {
    fn u<'b>(&self, e: &'b UExpr) -> UExprHelper<'b> {
        UExprHelper { p: self.p, e }
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

impl<'a> Display for UExprHelper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use UExpr::*;
        use Printer::*;
        match self.e {
            Ones => f.write_str("-1"),
            Var(c) => f.write_char(*c),
            Not(i) => {
                let i = self.u(i);
                match self.p {
                    C if i.e.is_unary() => write!(f, "~{}", i),
                    C => write!(f, "~({})", i),
                    Rust if i.e.is_unary() => write!(f, "!{}", i),
                    Rust => write!(f, "!({})", i),
                    Tex => write!(f, "\\overline{{{}}}", i),
                }
            },
            And(l, r) => {
                match self.p {
                    C | Rust => self.write_safe(l, r, "&", f),
                    Tex => self.write_safe(l, r, "\\land", f),
                }
            },
            Or(l, r) => {
                match self.p {
                    C | Rust => self.write_safe(l, r, "|", f),
                    Tex => self.write_safe(l, r, "\\lor", f),
                }
            },
            Xor(l, r) => {
                match self.p {
                    C | Rust => self.write_safe(l, r, "^", f),
                    Tex => self.write_safe(l, r, "\\oplus", f),
                }
            }
        }
    }
}