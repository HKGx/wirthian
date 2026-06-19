use crate::ast::{Expr, ExprKind, Program, Statement, StmtKind, Type};
use miette::{Diagnostic, LabeledSpan, NamedSource};
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("{message}")]
pub struct SemanticError {
    pub message: String,
    pub src: NamedSource<String>,
    pub labels: Vec<LabeledSpan>,
}

impl Diagnostic for SemanticError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        Some(Box::new(self.labels.iter().cloned()))
    }
}

pub struct CheckerResult {
    pub errors: Vec<SemanticError>,
}

struct RawError<'a> {
    pub message: String,
    pub span: Range<usize>,
    pub related_var: Option<&'a str>,
}

pub struct Checker<'a> {
    symbol_table: HashMap<&'a str, Type>,
    loop_depth: usize,

    errors: Vec<RawError<'a>>,
    var_occurrences: HashMap<&'a str, Vec<Range<usize>>>,
    misused_vars: HashSet<&'a str>,
}

impl<'a> Checker<'a> {
    pub fn new() -> Self {
        Checker {
            symbol_table: HashMap::new(),
            loop_depth: 0,
            errors: Vec::new(),
            var_occurrences: HashMap::new(),
            misused_vars: HashSet::new(),
        }
    }

    pub fn check_program(
        mut self,
        program: &Program<'a>,
        source: &str,
    ) -> Result<(), CheckerResult> {
        for decl in &program.declarations {
            if self.symbol_table.contains_key(decl.identifier) {
                self.errors.push(RawError {
                    message: format!("Ponowna deklaracja zmiennej: {}", decl.identifier),
                    span: decl.span.clone(),
                    related_var: Some(decl.identifier),
                });
                self.misused_vars.insert(decl.identifier);
            } else {
                self.symbol_table.insert(decl.identifier, decl.var_type);
                self.var_occurrences
                    .entry(decl.identifier)
                    .or_default()
                    .push(decl.span.clone());
            }
        }

        for instr in &program.instructions {
            self.check_statement(instr);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            let mut final_errors = Vec::new();

            let overlaps =
                |a: &Range<usize>, b: &Range<usize>| -> bool { a.start < b.end && b.start < a.end };

            for err in self.errors {
                let mut labels = Vec::new();

                labels.push(LabeledSpan::new(
                    Some(err.message.clone()),
                    err.span.start,
                    err.span.end - err.span.start,
                ));

                if let Some(var) = err.related_var {
                    if let Some(occurrences) = self.var_occurrences.get(var) {
                        let mut sorted_occurrences = occurrences.clone();
                        sorted_occurrences.sort_by_key(|s| s.start);

                        let mut accepted_spans: Vec<Range<usize>> = vec![err.span.clone()];

                        for occ_span in sorted_occurrences {
                            if !accepted_spans.iter().any(|s| overlaps(s, &occ_span)) {
                                labels.push(LabeledSpan::new(
                                    None,
                                    occ_span.start,
                                    occ_span.end - occ_span.start,
                                ));
                                accepted_spans.push(occ_span);
                            }
                        }
                    }
                }

                final_errors.push(SemanticError {
                    message: err.message,
                    src: NamedSource::new("source.pa", source.to_string()),
                    labels,
                });
            }

            Err(CheckerResult {
                errors: final_errors,
            })
        }
    }

    fn check_statement(&mut self, stmt: &Statement<'a>) {
        match &stmt.kind {
            StmtKind::Assign(id, expr) => {
                self.var_occurrences
                    .entry(*id)
                    .or_default()
                    .push(stmt.span.clone());

                let expr_type = self.check_expr(expr);
                if let Some(var_type) = self.symbol_table.get(id).copied() {
                    if let Some(et) = expr_type {
                        if var_type != et {
                            self.errors.push(RawError {
                                message: format!("Niezgodność typów: przypisujesz wartość typu {:?} do zmiennej '{}' typu {:?}", et, id, var_type),
                                span: expr.span.clone(),
                                related_var: Some(*id),
                            });
                            self.misused_vars.insert(*id);
                        }
                    }
                } else {
                    self.errors.push(RawError {
                        message: format!("Przypisanie do niezadeklarowanej zmiennej: {}", id),
                        span: stmt.span.clone(),
                        related_var: Some(*id),
                    });
                    self.misused_vars.insert(*id);
                }
            }
            StmtKind::If {
                condition,
                then_branch,
                elif_branches,
                else_branch,
            } => {
                self.expect_type(condition, Type::Boolean);
                self.check_statement(then_branch);
                for (elif_cond, elif_body) in elif_branches {
                    self.expect_type(elif_cond, Type::Boolean);
                    self.check_statement(elif_body);
                }
                if let Some(else_b) = else_branch {
                    self.check_statement(else_b);
                }
            }
            StmtKind::For {
                iterator,
                from,
                to,
                body,
            } => {
                self.var_occurrences
                    .entry(*iterator)
                    .or_default()
                    .push(stmt.span.clone());

                if let Some(var_type) = self.symbol_table.get(iterator).copied() {
                    if var_type != Type::Integer {
                        self.errors.push(RawError {
                            message: format!(
                                "Zmienna iterująca '{}' w pętli for musi być typu Integer",
                                iterator
                            ),
                            span: stmt.span.clone(),
                            related_var: Some(*iterator),
                        });
                        self.misused_vars.insert(*iterator);
                    }
                } else {
                    self.errors.push(RawError {
                        message: format!(
                            "Niezadeklarowana zmienna iterująca w pętli: {}",
                            iterator
                        ),
                        span: stmt.span.clone(),
                        related_var: Some(*iterator),
                    });
                    self.misused_vars.insert(*iterator);
                }

                self.expect_type(from, Type::Integer);
                self.expect_type(to, Type::Integer);

                self.loop_depth += 1;
                self.check_statement(body);
                self.loop_depth -= 1;
            }
            StmtKind::Block(stmts) => {
                for s in stmts {
                    self.check_statement(s);
                }
            }
            StmtKind::Print(expr) => {
                self.check_expr(expr);
            }
            StmtKind::Break => {
                if self.loop_depth == 0 {
                    self.errors.push(RawError {
                        message: "Instrukcja 'break' użyta poza pętlą".into(),
                        span: stmt.span.clone(),
                        related_var: None,
                    });
                }
            }
            StmtKind::Continue => {
                if self.loop_depth == 0 {
                    self.errors.push(RawError {
                        message: "Instrukcja 'continue' użyta poza pętlą".into(),
                        span: stmt.span.clone(),
                        related_var: None,
                    });
                }
            }
            StmtKind::Exit => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr<'a>) -> Option<Type> {
        match &expr.kind {
            ExprKind::Number(_) => Some(Type::Integer),
            ExprKind::StringLit(_) => Some(Type::String),
            ExprKind::True | ExprKind::False => Some(Type::Boolean),
            ExprKind::Identifier(id) => {
                self.var_occurrences
                    .entry(*id)
                    .or_default()
                    .push(expr.span.clone());
                if let Some(t) = self.symbol_table.get(id).copied() {
                    Some(t)
                } else {
                    self.errors.push(RawError {
                        message: format!("Niezadeklarowana zmienna: {}", id),
                        span: expr.span.clone(),
                        related_var: Some(*id),
                    });
                    self.misused_vars.insert(*id);
                    None
                }
            }
            ExprKind::Add(l, r)
            | ExprKind::Sub(l, r)
            | ExprKind::Mul(l, r)
            | ExprKind::Div(l, r)
            | ExprKind::Mod(l, r) => {
                self.expect_type(l, Type::Integer);
                self.expect_type(r, Type::Integer);
                Some(Type::Integer)
            }
            ExprKind::Concatenate(l, r) => {
                self.expect_type(l, Type::String);
                self.expect_type(r, Type::String);
                Some(Type::String)
            }
            ExprKind::Substring(s, p, len) => {
                self.expect_type(s, Type::String);
                self.expect_type(p, Type::Integer);
                self.expect_type(len, Type::Integer);
                Some(Type::String)
            }
            ExprKind::Length(s) => {
                self.expect_type(s, Type::String);
                Some(Type::Integer)
            }
            ExprKind::Position(s1, s2) => {
                self.expect_type(s1, Type::String);
                self.expect_type(s2, Type::String);
                Some(Type::Integer)
            }
            ExprKind::Not(e) => {
                self.expect_type(e, Type::Boolean);
                Some(Type::Boolean)
            }
            ExprKind::And(l, r) | ExprKind::Or(l, r) => {
                self.expect_type(l, Type::Boolean);
                self.expect_type(r, Type::Boolean);
                Some(Type::Boolean)
            }
            ExprKind::Eq(l, r) | ExprKind::Neq(l, r) => {
                let lt = self.check_expr(l);
                let rt = self.check_expr(r);
                if let (Some(lt_val), Some(rt_val)) = (lt, rt) {
                    if lt_val != rt_val {
                        let mut related_var = None;
                        if let ExprKind::Identifier(id) = &l.kind {
                            self.misused_vars.insert(*id);
                            related_var = Some(*id);
                        }
                        if let ExprKind::Identifier(id) = &r.kind {
                            self.misused_vars.insert(*id);
                            if related_var.is_none() {
                                related_var = Some(*id);
                            }
                        }

                        self.errors.push(RawError {
                            message: format!(
                                "Porównanie niezgodnych typów: {:?} oraz {:?}",
                                lt_val, rt_val
                            ),
                            span: expr.span.clone(),
                            related_var,
                        });
                    }
                }
                Some(Type::Boolean)
            }
            ExprKind::Less(l, r)
            | ExprKind::LessEq(l, r)
            | ExprKind::Greater(l, r)
            | ExprKind::GreaterEq(l, r) => {
                self.expect_type(l, Type::Integer);
                self.expect_type(r, Type::Integer);
                Some(Type::Boolean)
            }
            ExprKind::ReadInt => Some(Type::Integer),
            ExprKind::ReadStr => Some(Type::String),
            ExprKind::ReadBool => Some(Type::Boolean),
        }
    }

    fn expect_type(&mut self, expr: &Expr<'a>, expected: Type) -> Option<Type> {
        let actual = self.check_expr(expr)?;
        if actual != expected {
            let mut related_var = None;
            if let ExprKind::Identifier(id) = &expr.kind {
                self.misused_vars.insert(*id);
                related_var = Some(*id);
            }

            self.errors.push(RawError {
                message: format!(
                    "Niezgodność typów: oczekiwano {:?}, otrzymano {:?}",
                    expected, actual
                ),
                span: expr.span.clone(),
                related_var,
            });
            return None;
        }
        Some(actual)
    }
}
