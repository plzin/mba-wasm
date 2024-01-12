use std::rc::Rc;
use std::fmt::Write;
use std::collections::BTreeSet;
use crate::{numbers::{UnsignedInt, int_from_it}, printer::Printer};

#[derive(Debug, Clone)]
pub enum Expr<T> {
    Const(T),
    Var(String),

    // Arithmetic
    Add(Rc<Expr<T>>, Rc<Expr<T>>),
    Sub(Rc<Expr<T>>, Rc<Expr<T>>),
    Mul(Rc<Expr<T>>, Rc<Expr<T>>),
    Div(Rc<Expr<T>>, Rc<Expr<T>>),
    Mod(Rc<Expr<T>>, Rc<Expr<T>>),
    Neg(Rc<Expr<T>>),

    // Boolean
    And(Rc<Expr<T>>, Rc<Expr<T>>),
    Or(Rc<Expr<T>>, Rc<Expr<T>>),
    Xor(Rc<Expr<T>>, Rc<Expr<T>>),
    Shl(Rc<Expr<T>>, Rc<Expr<T>>),
    Shr(Rc<Expr<T>>, Rc<Expr<T>>),
    Not(Rc<Expr<T>>),
}

impl<T: UnsignedInt> Expr<T> {
    /// Returns the zero constant.
    pub fn zero() -> Self {
        Self::Const(T::zero())
    }

    /// Returns all variables in the expression.
    pub fn vars(&self) -> Vec<String> {
        let mut v = BTreeSet::new();
        self.vars_impl(&mut v);
        v.into_iter().collect()
    }

    fn vars_impl(&self, v: &mut BTreeSet<String>) {
        match self {
            Expr::Const(_) => {},
            Expr::Var(name) => drop(v.insert(name.clone())),
            Expr::Neg(e) | Expr::Not(e) => e.vars_impl(v),

            Expr::Add(l, r) | Expr::Sub(l, r) | Expr::Mul(l, r)
            | Expr::Div(l, r) | Expr::Mod(l, r) | Expr::And(l, r)
            | Expr::Or(l, r) | Expr::Xor(l, r) | Expr::Shl(l, r)
            | Expr::Shr(l, r) => {
                l.vars_impl(v);
                r.vars_impl(v);
            }
        }
    }

    /// Substitute and expression for a variable.
    pub fn substitute(&mut self, e: &mut Rc<Expr<T>>, var: &str) {
        let mut visited = Vec::new();
        match self {
            Expr::Const(_) => {},
            Expr::Var(v) => if v == var { *self = e.as_ref().clone() },
            Expr::Neg(i) | Expr::Not(i) => Self::substitute_impl(i, var, e, &mut visited),
            Expr::Add(l, r) | Expr::Sub(l, r) | Expr::Mul(l, r)
            | Expr::Div(l, r) | Expr::Mod(l, r) | Expr::And(l, r)
            | Expr::Or(l, r) | Expr::Xor(l, r) | Expr::Shl(l, r)
            | Expr::Shr(l, r) => {
                Self::substitute_impl(l, var, e, &mut visited);
                Self::substitute_impl(r, var, e, &mut visited);
            },
        }
    }

    fn substitute_impl(
        this: &mut Rc<Expr<T>>, var: &str,
        e: &mut Rc<Expr<T>>, visited: &mut Vec<*const Expr<T>>
    ) {
        let ptr = Rc::as_ptr(this);
        let recurse = if visited.contains(&ptr) {
            false
        } else {
            visited.push(ptr);
            true
        };

        use Expr::*;
        // SAFETY: This is okay because we make sure with extra logic
        // that this is never encountered twice.
        match unsafe { &mut *(ptr as *mut _) } {
            Const(_) => {},
            Var(v) => if v == var { *this = e.clone() },
            Add(l, r) | Sub(l, r) | Mul(l, r) | Div(l, r) | Mod(l, r)
            | And(l, r) | Or(l, r) | Xor(l, r) | Shl(l, r)
            | Shr(l, r) => if recurse {
                    Self::substitute_impl(l, var, e, visited);
                    Self::substitute_impl(r, var, e, visited);
            },
            Neg(i) | Not(i) => if recurse {
                Self::substitute_impl(i, var, e, visited)
            },
        }
    }

    /// Returns the precedence of a binary operator.
    /// All operators are treated as being left associative.
    fn precedence(&self) -> usize {
        use Expr::*;
        match self {
            Or(_, _) => 1,
            Xor(_, _) => 2,
            And(_, _) => 3,
            Shl(_, _) | Shr(_, _) => 4,
            Add(_, _) | Sub(_, _) => 5,
            Mul(_, _) | Div(_, _) | Mod(_, _) => 6,
            Neg(_) | Not(_) => 255,
            Const(_) | Var(_) => 256,
        }
    }

    /// Parse an expression from a string.
    /// Can't parse shifts at the moment.
    /// Closing brackets are a bit broken.
    pub fn from_string<U: ToString>(s: U) -> Result<Expr<T>, String> {
        let mut s = s.to_string();
        s.retain(|c| !c.is_whitespace());
        let mut it = s.chars().peekable();

        Self::parse(&mut it, 0)
    }

    // pre 0: parse as much as possible
    // ...
    // pre 15: parse as little as possible
    fn parse(
        it: &mut std::iter::Peekable<std::str::Chars>,
        pre: usize
    ) -> Result<Self, String> {
        use Expr::*;

        let mut c = *it.peek()
            .ok_or_else(|| "Unexpected end of input".to_owned())?;

        let mut e = if c == '(' {
            it.next();
            let e = Self::parse(it, 0)?;
            match it.next() {
                Some(')') => e,
                _ => return Err("Closing bracket missing".into()),
            }
        } else if c == '~' {
            it.next();
            let e = Self::parse(it, 15)?;
            Not(Rc::new(e))
        } else if c == '-' {
            it.next();
            let e = Self::parse(it, 15)?;
            Neg(Rc::new(e))
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
        } else if c.is_ascii_digit() {
            // This can't panic because we check that
            // the character is an ascii digit.
            let num = int_from_it(it).unwrap();
            Const(num)
        } else {
            return Err("Unrecognized character".into());
        };

        loop {
            let Some(c) = it.peek().cloned() else {
                return Ok(e)
            };

            let op_pre = match c {
                '|' => 1,
                '^' => 2,
                '&' => 3,
                '+' | '-' => 5,
                '*' | '/' | '%' => 6,
                ')' => return Ok(e),
                _ => return Err("Unknown operator".into()),
            };

            if op_pre <= pre {
                return Ok(e);
            }

            // If the current operators precedence is higher than
            // the one whose subexpression we are currently parsing
            // then we need to finish this operator first.
            it.next();
            let rhs = Rc::new(Self::parse(it, op_pre)?);
            let lhs = Rc::new(e);
            e = match c {
                '+' => Add(lhs, rhs),
                '-' => Sub(lhs, rhs),
                '*' => Mul(lhs, rhs),
                '/' => Div(lhs, rhs),
                '%' => Mod(lhs, rhs),
                '&' => And(lhs, rhs),
                '|' => Or(lhs, rhs),
                '^' => Xor(lhs, rhs),
                _ => unreachable!(),
            };
        };
    }

    /// Prints the expression while avoiding reprinting
    /// common subexpressions by assigning them to variables.
    /// This only works if the Rc's used in the expression
    /// are not shared with other expressions.
    pub fn print_as_fn(&self, printer: Printer) -> String {
        assert!(printer != Printer::Tex,
            "Tex printing is not supported for general expressions.");

        let mut input = self.vars();
        // This shouldn't really be done here,
        // but I can't be bothered.
        input.sort_by(|l, r| {
            if l.starts_with("aux") {
                if r.starts_with("aux") {
                    l.cmp(r)
                } else {
                    std::cmp::Ordering::Greater
                }
            } else if r.starts_with("aux") {
                std::cmp::Ordering::Less
            } else {
                l.cmp(r)
            }
        });

        // Stores a mapping of (sub)expressions to variables.
        let mut vars = Vec::new();

        let l = self.print_simple_impl(&mut vars, printer);

        let mut s = String::new();
        if printer == Printer::Default {
            for (_, var, init) in vars.iter().rev() {
                write!(&mut s, "{} = {}", var, init);
            }
        } else if printer == Printer::C {
            let ty = match std::mem::size_of::<T>() {
                1 => "uint8_t",
                2 => "uint16_t",
                4 => "uint32_t",
                8 => "uint64_t",
                16 => "uint128_t",
                _ => panic!("Unknown type."),
            };

            write!(&mut s, "{} f(", ty);
            for v in &input {
                write!(&mut s, "{} {}, ", ty, v);
            }
            s.pop();
            s.pop();
            write!(&mut s, ") {{\n");

            for (_, var, init) in vars.iter().rev() {
                write!(&mut s, "\t{} {} = {};\n", ty, var, init);
            }

            write!(&mut s, "\treturn {};\n}}", &l);
        } else if printer == Printer::Rust {
            let ty = match std::mem::size_of::<T>() {
                1 => "Wrapping<u8>",
                2 => "Wrapping<u16>",
                4 => "Wrapping<u32>",
                8 => "Wrapping<u64>",
                16 => "Wrapping<u128>",
                _ => panic!("Unknown type."),
            };

            write!(&mut s, "fn f(");
            for v in &input {
                write!(&mut s, "{}: {}, ", v, ty);
            }
            s.pop();
            s.pop();
            write!(&mut s, ") -> {} {{\n", ty);

            for (_, var, init) in vars.iter().rev() {
                write!(&mut s, "\tlet {} = {};\n", var, init);
            }

            write!(&mut s, "\t{}\n}}", &l);
        } else {
            assert!(false, "Unsupported printer.");
        }

        s
    }

    fn print_simple_rc(
        e: &Rc<Self>,
        vars: &mut Vec<(*const Self, String, String)>,
        printer: Printer
    ) -> String {
        // If there is only one reference then just print it.
        if Rc::strong_count(e) == 1 {
            return e.print_simple_impl(vars, printer);
        }

        // We don't want to assign a variable to a variable
        // so there is this shortcut here.
        if let Expr::Var(v) = &**e {
            return format!("{}", *v);
        }

        let ptr = Rc::as_ptr(e);

        // If the expression already has a variable then just print the variable.
        let var = vars.iter().find(|t| t.0 == ptr);
        if let Some(v) = var {
            v.1.clone()
        } else {
            let v = format!("var{}", vars.len());

            // Push everything.
            vars.push((ptr, v.clone(), String::new()));

            let idx = vars.len() - 1;

            // Get the initializer for the variable.
            vars[idx].2 = e.print_simple_impl(vars, printer);

            // Return just the variable name.
            v
        }
    }

    // Yes, this PERFORMANCE CRITICAL code could be more efficient...
    fn print_simple_impl(
        &self,
        vars: &mut Vec<(*const Self, String, String)>,
        printer: Printer
    ) -> String {
        // Print a binary operation.
        let bin_op = |
            op: &str, l: &Rc<Self>, r: &Rc<Self>,
            vars: &mut Vec<(*const Self, String, String)>
        | {
            let pred = self.precedence();

            let l = if pred > l.precedence() && Rc::strong_count(l) == 1 {
                format!("({})", Self::print_simple_rc(l, vars, printer))
            } else {
                format!("{}", Self::print_simple_rc(l, vars, printer))
            };

            let r = if pred > r.precedence() && Rc::strong_count(r) == 1 {
                format!("({})", Self::print_simple_rc(r, vars, printer))
            } else {
                format!("{}", Self::print_simple_rc(r, vars, printer))
            };

            format!("{} {} {}", l, op, r)
        };

        // Print a unary operation.
        let un_op = |
            op: &str, i: &Rc<Self>,
            vars: &mut Vec<(*const Self, String, String)>
        | {
            if self.precedence() > i.precedence() && Rc::strong_count(i) == 1 {
                format!("{}({})", op, Self::print_simple_rc(i, vars, printer))
            } else {
                format!("{}{}", op, Self::print_simple_rc(i, vars, printer))
            }
        };

        use Expr::*;
        match self {
            Const(i) if printer == Printer::Rust => format!("Wrapping({})", i),
            Const(i) => format!("{}", i),
            Var(n) => format!("{}", n),
            Add(l, r) => bin_op("+", l, r, vars),
            Sub(l, r) => bin_op("-", l, r, vars),
            Mul(l, r) => bin_op("*", l, r, vars),
            Div(l, r) => bin_op("/", l, r, vars),
            Mod(l, r) => bin_op("%", l, r, vars),
            Neg(i) => un_op("-", i, vars),
            And(l, r) => bin_op("&", l, r, vars),
            Or(l, r) => bin_op("|", l, r, vars),
            Xor(l, r) => bin_op("^", l, r, vars),
            Shl(l, r) => bin_op("<<", l, r, vars),
            Shr(l, r) => bin_op(">>", l, r, vars),
            Not(i) if printer == Printer::Rust => un_op("!", i, vars),
            Not(i) => un_op("~", i, vars),
        }
    }
}