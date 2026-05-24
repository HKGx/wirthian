use fastrand::Rng;

struct ProgramGenerator {
    rng: Rng,
    symbols: Vec<Symbol>,
    next_symbol: usize,
}

#[derive(Clone)]
struct Symbol {
    name: String,
    ty: Type,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Type {
    Integer,
    String,
    Boolean,
}

impl ProgramGenerator {
    fn new(len: usize) -> Self {
        Self {
            rng: Rng::with_seed(len as u64),
            symbols: Vec::new(),
            next_symbol: 0,
        }
    }

    fn parseable_program(&mut self, target_bytes: usize) -> String {
        let mut source = String::with_capacity(target_bytes + 512);

        for _ in 0..3 {
            for ty in [Type::Integer, Type::String, Type::Boolean] {
                self.declaration(&mut source, ty);
            }
        }

        while source.len() < target_bytes {
            self.parseable_instruction(&mut source, 0);
        }

        source
    }

    fn parseable_instruction(&mut self, source: &mut String, depth: usize) {
        self.parseable_statement(source, depth);
        source.push_str(";\n");
    }

    fn parseable_statement(&mut self, source: &mut String, depth: usize) {
        let roll = if depth >= 2 {
            self.rng.usize(..75)
        } else {
            self.rng.usize(..100)
        };

        match roll {
            0..=42 => self.parseable_assignment(source),
            43..=66 => self.parseable_print(source),
            67..=74 => self.parseable_begin_block(source, depth),
            75..=89 => self.parseable_loop_statement(source, depth),
            _ => self.parseable_if_statement(source, depth),
        }
    }

    fn parseable_assignment(&mut self, source: &mut String) {
        let symbol = self.symbol().clone();
        source.push_str(&symbol.name);
        source.push_str(" := ");
        self.expression(source, symbol.ty, 0);
    }

    fn parseable_print(&mut self, source: &mut String) {
        source.push_str("print(");
        let ty = self.random_type();
        self.expression(source, ty, 0);
        source.push(')');
    }

    fn parseable_begin_block(&mut self, source: &mut String, depth: usize) {
        source.push_str("begin\n");
        for _ in 0..self.rng.usize(1..4) {
            self.parseable_instruction(source, depth + 1);
        }
        source.push_str("end");
    }

    fn parseable_loop_statement(&mut self, source: &mut String, depth: usize) {
        let counter = self.fresh_ident();
        self.symbols.push(Symbol {
            name: counter.clone(),
            ty: Type::Integer,
        });

        source.push_str("for ");
        source.push_str(&counter);
        source.push_str(" := ");
        self.integer_expression(source, 0);
        source.push_str(" to ");
        self.integer_expression(source, 0);
        source.push_str(" do ");
        self.parseable_begin_block(source, depth + 1);
    }

    fn parseable_if_statement(&mut self, source: &mut String, depth: usize) {
        source.push_str("if ");
        self.boolean_expression(source, 0);
        source.push_str(" then ");
        self.parseable_begin_block(source, depth + 1);

        if self.chance(1, 3) {
            source.push_str(" elif ");
            self.boolean_expression(source, 0);
            source.push_str(" then ");
            self.parseable_begin_block(source, depth + 1);
        }

        if self.chance(1, 3) {
            source.push_str(" elif ");
            self.boolean_expression(source, 0);
            source.push_str(" then ");
            self.parseable_begin_block(source, depth + 1);
        }

        source.push_str(" else ");
        self.parseable_begin_block(source, depth + 1);
    }

    fn declaration(&mut self, source: &mut String, ty: Type) {
        let name = self.fresh_ident();
        source.push_str(ty.keyword());
        source.push(' ');
        source.push_str(&name);
        source.push_str(";\n");
        self.symbols.push(Symbol { name, ty });
    }

    fn chance(&mut self, nominator: usize, denominator: usize) -> bool {
        nominator < self.rng.usize(..denominator)
    }

    fn expression(&mut self, source: &mut String, ty: Type, depth: usize) {
        match ty {
            Type::Integer => self.integer_expression(source, depth),
            Type::String => self.string_expression(source, depth),
            Type::Boolean => self.boolean_expression(source, depth),
        }
    }

    fn integer_expression(&mut self, source: &mut String, depth: usize) {
        if depth >= 3 {
            self.integer_atom(source);
            return;
        }

        match self.rng.usize(..100) {
            0..=44 => self.integer_atom(source),
            45..=73 => {
                source.push('(');
                self.integer_expression(source, depth + 1);
                source.push(' ');
                source.push_str(self.rng.choice(&["+", "-", "*", "/", "%"]).unwrap());
                source.push(' ');
                self.integer_expression(source, depth + 1);
                source.push(')');
            }
            74..=84 => {
                source.push_str("length(");
                self.string_expression(source, depth + 1);
                source.push(')');
            }
            _ => {
                source.push_str("position(");
                self.string_expression(source, depth + 1);
                source.push_str(", ");
                self.string_expression(source, depth + 1);
                source.push(')');
            }
        }
    }

    fn integer_atom(&mut self, source: &mut String) {
        if self.chance(2, 5) {
            source.push_str(&self.symbol_of_type(Type::Integer).name);
        } else {
            source.push_str(&self.rng.usize(..100_000).to_string());
        }
    }

    fn string_expression(&mut self, source: &mut String, depth: usize) {
        if depth >= 3 {
            self.string_atom(source);
            return;
        }

        match self.rng.usize(..100) {
            0..=49 => self.string_atom(source),
            50..=74 => {
                source.push_str("concatenate(");
                self.string_expression(source, depth + 1);
                source.push_str(", ");
                self.string_expression(source, depth + 1);
                source.push(')');
            }
            _ => {
                source.push_str("substring(");
                self.string_expression(source, depth + 1);
                source.push_str(", ");
                self.integer_expression(source, depth + 1);
                source.push_str(", ");
                self.integer_expression(source, depth + 1);
                source.push(')');
            }
        }
    }

    fn string_atom(&mut self, source: &mut String) {
        if self.chance(1, 3) {
            source.push_str(&self.symbol_of_type(Type::String).name);
        } else {
            source.push_str(
                self.rng
                    .choice(&[
                        r#""hello""#,
                        r#""line\nnext""#,
                        r#""tab\tvalue""#,
                        r#""quote\"inside""#,
                        r#""unicode\u0041""#,
                        r#""path\\file""#,
                    ])
                    .unwrap(),
            );
        }
    }

    fn boolean_expression(&mut self, source: &mut String, depth: usize) {
        if depth >= 3 {
            self.boolean_atom(source);
            return;
        }

        match self.rng.usize(..100) {
            0..=23 => self.boolean_atom(source),
            24..=43 => {
                source.push_str("not ");
                self.boolean_expression(source, depth + 1);
            }
            44..=63 => {
                source.push('(');
                self.boolean_expression(source, depth + 1);
                source.push(' ');
                source.push_str(self.rng.choice(&["and", "or"]).unwrap());
                source.push(' ');
                self.boolean_expression(source, depth + 1);
                source.push(')');
            }
            64..=83 => {
                self.integer_expression(source, depth + 1);
                source.push(' ');
                source.push_str(self.rng.choice(&["=", "<", "<=", ">", ">=", "<>"]).unwrap());
                source.push(' ');
                self.integer_expression(source, depth + 1);
            }
            _ => {
                self.string_expression(source, depth + 1);
                source.push(' ');
                source.push_str(self.rng.choice(&["==", "!="]).unwrap());
                source.push(' ');
                self.string_expression(source, depth + 1);
            }
        }
    }

    fn boolean_atom(&mut self, source: &mut String) {
        match self.rng.usize(..3) {
            0 => source.push_str("true"),
            1 => source.push_str("false"),
            _ => source.push_str(&self.symbol_of_type(Type::Boolean).name),
        }
    }

    fn fresh_ident(&mut self) -> String {
        let prefix = self
            .rng
            .choice(&[
                "count", "index", "value", "flag", "name", "input", "total", "result", "buffer",
                "item",
            ])
            .unwrap();

        let ident = format!("{}_{}", prefix, self.next_symbol);
        self.next_symbol += 1;
        ident
    }

    fn symbol(&mut self) -> &Symbol {
        let index = self.rng.usize(..self.symbols.len());
        &self.symbols[index]
    }

    fn symbol_of_type(&mut self, ty: Type) -> Symbol {
        let matches: Vec<_> = self
            .symbols
            .iter()
            .filter(|symbol| symbol.ty == ty)
            .cloned()
            .collect();

        if matches.is_empty() {
            return Symbol {
                name: match ty {
                    Type::Integer => "0".to_string(),
                    Type::String => r#""fallback""#.to_string(),
                    Type::Boolean => "true".to_string(),
                },
                ty,
            };
        }

        matches[self.rng.usize(..matches.len())].clone()
    }

    fn random_type(&mut self) -> Type {
        match self.rng.usize(..3) {
            0 => Type::Integer,
            1 => Type::String,
            _ => Type::Boolean,
        }
    }
}

impl Type {
    fn keyword(self) -> &'static str {
        match self {
            Type::Integer => "integer",
            Type::String => "string",
            Type::Boolean => "boolean",
        }
    }
}

pub fn generated_program(bytes: usize) -> String {
    ProgramGenerator::new(bytes).parseable_program(bytes)
}
