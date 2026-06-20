use crate::ast::{Expr, ExprKind, Program, Statement, StmtKind, Type};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i32),
    Str(Rc<str>),
    Boolean(bool),
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("dzielenie przez zero")]
    DivisionByZero,
    #[error("niezgodność typów")]
    TypeMismatch,
    #[error("błąd wejścia/wyjścia: {0}")]
    Io(#[from] std::io::Error),
}

enum Flow {
    Normal,
    Break,
    Continue,
    Exit,
}

pub struct Interpreter<'a, R, W> {
    env: Vec<Value>,
    names: Vec<&'a str>,
    var_types: Vec<Type>,
    string_cache: HashMap<&'a str, Rc<str>>,
    input: R,
    output: W,
}

impl<'a, R: BufRead, W: Write> Interpreter<'a, R, W> {
    pub fn new(program: &Program<'a>, input: R, output: W) -> Self {
        let mut names = Vec::with_capacity(program.declarations.len());
        let mut env = Vec::with_capacity(program.declarations.len());
        let mut var_types = Vec::with_capacity(program.declarations.len());

        for decl in &program.declarations {
            names.push(decl.identifier);
            var_types.push(decl.var_type);
            env.push(match decl.var_type {
                Type::Integer => Value::Integer(0),
                Type::String => Value::Str(Rc::from("")),
                Type::Boolean => Value::Boolean(false),
            });
        }

        Interpreter {
            env,
            names,
            var_types,
            string_cache: HashMap::new(),
            input,
            output,
        }
    }

    pub fn run(&mut self, program: &Program<'a>) -> Result<(), RuntimeError> {
        for stmt in &program.instructions {
            match self.exec(stmt)? {
                Flow::Exit => return Ok(()),
                _ => {}
            }
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }

    fn slot(&self, name: &'a str) -> usize {
        self.names
            .iter()
            .position(|n| *n == name)
            .unwrap_or_else(|| panic!("undeclared variable: {name}"))
    }

    fn exec(&mut self, stmt: &Statement<'a>) -> Result<Flow, RuntimeError> {
        match &stmt.kind {
            StmtKind::Assign(id, expr) => {
                let slot = self.slot(id);
                let val = match self.var_types[slot] {
                    Type::Integer => Value::Integer(self.eval_int(expr)?),
                    Type::String => Value::Str(self.eval_str(expr)?),
                    Type::Boolean => Value::Boolean(self.eval_bool(expr)?),
                };
                self.env[slot] = val;
                Ok(Flow::Normal)
            }
            StmtKind::If {
                condition,
                then_branch,
                elif_branches,
                else_branch,
            } => {
                if self.eval_bool(condition)? {
                    return self.exec(then_branch);
                }

                for (cond, body) in elif_branches {
                    if self.eval_bool(cond)? {
                        return self.exec(body);
                    }
                }

                if let Some(else_b) = else_branch {
                    return self.exec(else_b);
                }

                Ok(Flow::Normal)
            }
            StmtKind::For {
                iterator,
                from,
                to,
                body,
            } => {
                let mut cur = self.eval_int(from)?;
                let to_val = self.eval_int(to)?;
                let slot = self.slot(iterator);

                while cur <= to_val {
                    self.env[slot] = Value::Integer(cur);

                    match self.exec(body)? {
                        Flow::Normal | Flow::Continue => {}
                        Flow::Break => break,
                        Flow::Exit => return Ok(Flow::Exit),
                    }

                    match cur.checked_add(1) {
                        Some(next) => cur = next,
                        None => break,
                    }
                }

                self.env[slot] = Value::Integer(cur);

                Ok(Flow::Normal)
            }
            StmtKind::Block(stmts) => {
                for s in stmts {
                    match self.exec(s)? {
                        Flow::Normal => {}
                        flow => return Ok(flow),
                    }
                }

                Ok(Flow::Normal)
            }
            StmtKind::Print(expr) => {
                let val = self.eval(expr)?;

                match val {
                    Value::Integer(n) => write!(self.output, "{n}")?,
                    Value::Str(s) => self.output.write_all(s.as_bytes())?,
                    Value::Boolean(b) => write!(self.output, "{b}")?,
                }

                Ok(Flow::Normal)
            }
            StmtKind::Break => Ok(Flow::Break),
            StmtKind::Continue => Ok(Flow::Continue),
            StmtKind::Exit => Ok(Flow::Exit),
        }
    }

    fn eval(&mut self, expr: &Expr<'a>) -> Result<Value, RuntimeError> {
        match &expr.kind {
            ExprKind::Number(n) => Ok(Value::Integer(*n)),
            ExprKind::True => Ok(Value::Boolean(true)),
            ExprKind::False => Ok(Value::Boolean(false)),
            ExprKind::StringLit(s) => {
                let v = if let Some(existing) = self.string_cache.get(s) {
                    Rc::clone(existing)
                } else {
                    let unescaped = Rc::from(unescape_string(s).as_str());
                    self.string_cache.insert(s, Rc::clone(&unescaped));
                    unescaped
                };

                Ok(Value::Str(v))
            }
            ExprKind::Identifier(id) => {
                let slot = self.slot(id);

                Ok(self.env[slot].clone())
            }
            ExprKind::ReadInt => {
                let mut line = String::new();

                if self.input.read_line(&mut line)? == 0 {
                    return Ok(Value::Integer(0));
                }

                let val = line.trim().parse::<i32>().unwrap_or(0);

                Ok(Value::Integer(val))
            }
            ExprKind::ReadStr => {
                let mut line = String::new();

                if self.input.read_line(&mut line)? == 0 {
                    return Ok(Value::Str(Rc::from("")));
                }

                let s = line.strip_suffix('\n').unwrap_or(&line);
                let s = s.strip_suffix('\r').unwrap_or(s);

                Ok(Value::Str(Rc::from(s)))
            }
            ExprKind::ReadBool => {
                let mut line = String::new();

                if self.input.read_line(&mut line)? == 0 {
                    return Ok(Value::Boolean(false));
                }

                Ok(Value::Boolean(line.trim() == "true"))
            }
            ExprKind::Add(a, b) => Ok(Value::Integer(
                self.eval_int(a)?.wrapping_add(self.eval_int(b)?),
            )),
            ExprKind::Sub(a, b) => Ok(Value::Integer(
                self.eval_int(a)?.wrapping_sub(self.eval_int(b)?),
            )),
            ExprKind::Mul(a, b) => Ok(Value::Integer(
                self.eval_int(a)?.wrapping_mul(self.eval_int(b)?),
            )),
            ExprKind::Div(a, b) => {
                let r = self.eval_int(b)?;

                if r == 0 {
                    return Err(RuntimeError::DivisionByZero);
                }

                Ok(Value::Integer(self.eval_int(a)?.wrapping_div(r)))
            }
            ExprKind::Mod(a, b) => {
                let r = self.eval_int(b)?;

                if r == 0 {
                    return Err(RuntimeError::DivisionByZero);
                }

                Ok(Value::Integer(self.eval_int(a)?.wrapping_rem(r)))
            }
            ExprKind::Concatenate(a, b) => {
                let av = self.eval_str(a)?;
                let bv = self.eval_str(b)?;

                let mut combined = String::with_capacity(av.len() + bv.len());
                combined.push_str(&av);
                combined.push_str(&bv);

                Ok(Value::Str(Rc::from(combined.as_str())))
            }
            ExprKind::Substring(s, pos, len) => {
                let sv = self.eval_str(s)?;
                let pos = self.eval_int(pos)?;
                let len = self.eval_int(len)?;

                Ok(Value::Str(substring(&sv, pos, len)))
            }
            ExprKind::Length(s) => {
                let sv = self.eval_str(s)?;

                Ok(Value::Integer(sv.chars().count() as i32))
            }
            ExprKind::Position(hay, needle) => {
                let hv = self.eval_str(hay)?;
                let nv = self.eval_str(needle)?;

                Ok(Value::Integer(position(&hv, &nv)))
            }
            ExprKind::Not(a) => Ok(Value::Boolean(!self.eval_bool(a)?)),
            ExprKind::And(a, b) => {
                if !self.eval_bool(a)? {
                    return Ok(Value::Boolean(false));
                }

                Ok(Value::Boolean(self.eval_bool(b)?))
            }
            ExprKind::Or(a, b) => {
                if self.eval_bool(a)? {
                    return Ok(Value::Boolean(true));
                }

                Ok(Value::Boolean(self.eval_bool(b)?))
            }
            ExprKind::Eq(a, b) => {
                let l = self.eval(a)?;
                let r = self.eval(b)?;

                Ok(Value::Boolean(value_eq(&l, &r)))
            }
            ExprKind::Neq(a, b) => {
                let l = self.eval(a)?;
                let r = self.eval(b)?;

                Ok(Value::Boolean(!value_eq(&l, &r)))
            }
            ExprKind::Less(a, b) => Ok(Value::Boolean(self.eval_int(a)? < self.eval_int(b)?)),
            ExprKind::LessEq(a, b) => Ok(Value::Boolean(self.eval_int(a)? <= self.eval_int(b)?)),
            ExprKind::Greater(a, b) => Ok(Value::Boolean(self.eval_int(a)? > self.eval_int(b)?)),
            ExprKind::GreaterEq(a, b) => Ok(Value::Boolean(self.eval_int(a)? >= self.eval_int(b)?)),
        }
    }

    fn eval_int(&mut self, expr: &Expr<'a>) -> Result<i32, RuntimeError> {
        match &expr.kind {
            ExprKind::Number(n) => Ok(*n),
            ExprKind::Identifier(id) => {
                let slot = self.slot(id);
                match self.env[slot] {
                    Value::Integer(n) => Ok(n),
                    _ => Err(RuntimeError::TypeMismatch),
                }
            }
            ExprKind::Add(a, b) => Ok(self.eval_int(a)?.wrapping_add(self.eval_int(b)?)),
            ExprKind::Sub(a, b) => Ok(self.eval_int(a)?.wrapping_sub(self.eval_int(b)?)),
            ExprKind::Mul(a, b) => Ok(self.eval_int(a)?.wrapping_mul(self.eval_int(b)?)),
            ExprKind::Div(a, b) => {
                let r = self.eval_int(b)?;
                if r == 0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                Ok(self.eval_int(a)?.wrapping_div(r))
            }
            ExprKind::Mod(a, b) => {
                let r = self.eval_int(b)?;
                if r == 0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                Ok(self.eval_int(a)?.wrapping_rem(r))
            }
            ExprKind::Length(s) => Ok(self.eval_str(s)?.chars().count() as i32),
            ExprKind::Position(hay, needle) => {
                Ok(position(&self.eval_str(hay)?, &self.eval_str(needle)?))
            }
            ExprKind::ReadInt => {
                let mut line = String::new();
                if self.input.read_line(&mut line)? == 0 {
                    return Ok(0);
                }
                Ok(line.trim().parse::<i32>().unwrap_or(0))
            }
            _ => match self.eval(expr)? {
                Value::Integer(n) => Ok(n),
                _ => Err(RuntimeError::TypeMismatch),
            },
        }
    }

    fn eval_bool(&mut self, expr: &Expr<'a>) -> Result<bool, RuntimeError> {
        match &expr.kind {
            ExprKind::True => Ok(true),
            ExprKind::False => Ok(false),
            ExprKind::Identifier(id) => {
                let slot = self.slot(id);
                match self.env[slot] {
                    Value::Boolean(b) => Ok(b),
                    _ => Err(RuntimeError::TypeMismatch),
                }
            }
            ExprKind::Not(a) => Ok(!self.eval_bool(a)?),
            ExprKind::And(a, b) => {
                if !self.eval_bool(a)? {
                    return Ok(false);
                }
                Ok(self.eval_bool(b)?)
            }
            ExprKind::Or(a, b) => {
                if self.eval_bool(a)? {
                    return Ok(true);
                }
                Ok(self.eval_bool(b)?)
            }
            ExprKind::Less(a, b) => Ok(self.eval_int(a)? < self.eval_int(b)?),
            ExprKind::LessEq(a, b) => Ok(self.eval_int(a)? <= self.eval_int(b)?),
            ExprKind::Greater(a, b) => Ok(self.eval_int(a)? > self.eval_int(b)?),
            ExprKind::GreaterEq(a, b) => Ok(self.eval_int(a)? >= self.eval_int(b)?),
            ExprKind::ReadBool => {
                let mut line = String::new();
                if self.input.read_line(&mut line)? == 0 {
                    return Ok(false);
                }
                Ok(line.trim() == "true")
            }
            _ => match self.eval(expr)? {
                Value::Boolean(b) => Ok(b),
                _ => Err(RuntimeError::TypeMismatch),
            },
        }
    }

    fn eval_str(&mut self, expr: &Expr<'a>) -> Result<Rc<str>, RuntimeError> {
        match &expr.kind {
            ExprKind::StringLit(s) => {
                if let Some(existing) = self.string_cache.get(s) {
                    return Ok(Rc::clone(existing));
                }
                let unescaped = Rc::from(unescape_string(s).as_str());
                self.string_cache.insert(s, Rc::clone(&unescaped));
                Ok(unescaped)
            }
            ExprKind::Identifier(id) => {
                let slot = self.slot(id);
                match &self.env[slot] {
                    Value::Str(s) => Ok(Rc::clone(s)),
                    _ => Err(RuntimeError::TypeMismatch),
                }
            }
            ExprKind::Concatenate(a, b) => {
                let av = self.eval_str(a)?;
                let bv = self.eval_str(b)?;
                let mut combined = String::with_capacity(av.len() + bv.len());
                combined.push_str(&av);
                combined.push_str(&bv);
                Ok(Rc::from(combined.as_str()))
            }
            ExprKind::Substring(s, pos, len) => {
                let sv = self.eval_str(s)?;
                Ok(substring(&sv, self.eval_int(pos)?, self.eval_int(len)?))
            }
            ExprKind::ReadStr => {
                let mut line = String::new();
                if self.input.read_line(&mut line)? == 0 {
                    return Ok(Rc::from(""));
                }
                let s = line.strip_suffix('\n').unwrap_or(&line);
                let s = s.strip_suffix('\r').unwrap_or(s);
                Ok(Rc::from(s))
            }
            _ => match self.eval(expr)? {
                Value::Str(s) => Ok(s),
                _ => Err(RuntimeError::TypeMismatch),
            },
        }
    }
}

fn value_eq(l: &Value, r: &Value) -> bool {
    match (l, r) {
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Str(a), Value::Str(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        _ => false,
    }
}

fn substring(s: &str, pos: i32, len: i32) -> Rc<str> {
    if pos < 1 || len <= 0 {
        return Rc::from("");
    }

    let start_idx = (pos - 1) as usize;
    let max_len = len as usize;
    let mut byte_start = s.len();
    let mut byte_end = s.len();
    let mut char_idx = 0usize;

    for (bi, _) in s.char_indices() {
        if char_idx == start_idx {
            byte_start = bi;
        }
        if char_idx == start_idx + max_len {
            byte_end = bi;
            break;
        }
        char_idx += 1;
    }
    Rc::from(&s[byte_start..byte_end])
}

fn position(hay: &str, needle: &str) -> i32 {
    match hay.find(needle) {
        Some(byte_pos) => (hay[..byte_pos].chars().count() + 1) as i32,
        None => 0,
    }
}

fn unescape_string(raw: &str) -> String {
    let inner = &raw[1..raw.len().saturating_sub(1)];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('b') => out.push('\u{0008}'),
                Some('f') => out.push('\u{000C}'),
                Some('r') => out.push('\r'),
                Some('/') => out.push('/'),
                Some('u') => {
                    let mut code = String::with_capacity(4);
                    for _ in 0..4 {
                        if let Some(h) = chars.next() {
                            code.push(h);
                        }
                    }
                    if let Ok(n) = u32::from_str_radix(&code, 16) {
                        if let Some(ch) = char::from_u32(n) {
                            out.push(ch);
                        }
                    }
                }
                _ => {}
            }
        } else {
            out.push(c);
        }
    }

    out
}
