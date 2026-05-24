use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Integer,
    String,
    Boolean,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr<'source> {
    pub kind: ExprKind<'source>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind<'source> {
    Number(i32),
    StringLit(&'source str),
    Identifier(&'source str),
    True,
    False,

    Add(Box<Expr<'source>>, Box<Expr<'source>>),
    Sub(Box<Expr<'source>>, Box<Expr<'source>>),
    Mul(Box<Expr<'source>>, Box<Expr<'source>>),
    Div(Box<Expr<'source>>, Box<Expr<'source>>),
    Mod(Box<Expr<'source>>, Box<Expr<'source>>),

    Concatenate(Box<Expr<'source>>, Box<Expr<'source>>),
    Substring(Box<Expr<'source>>, Box<Expr<'source>>, Box<Expr<'source>>),
    Length(Box<Expr<'source>>),
    Position(Box<Expr<'source>>, Box<Expr<'source>>),

    Not(Box<Expr<'source>>),
    And(Box<Expr<'source>>, Box<Expr<'source>>),
    Or(Box<Expr<'source>>, Box<Expr<'source>>),
    Eq(Box<Expr<'source>>, Box<Expr<'source>>),
    Neq(Box<Expr<'source>>, Box<Expr<'source>>),
    Less(Box<Expr<'source>>, Box<Expr<'source>>),
    LessEq(Box<Expr<'source>>, Box<Expr<'source>>),
    Greater(Box<Expr<'source>>, Box<Expr<'source>>),
    GreaterEq(Box<Expr<'source>>, Box<Expr<'source>>),

    ReadInt,
    ReadStr,
    ReadBool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement<'source> {
    pub kind: StmtKind<'source>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind<'source> {
    Assign(&'source str, Expr<'source>),
    If {
        condition: Expr<'source>,
        then_branch: Box<Statement<'source>>,
        elif_branches: Vec<(Expr<'source>, Statement<'source>)>,
        else_branch: Option<Box<Statement<'source>>>,
    },
    For {
        iterator: &'source str,
        from: Expr<'source>,
        to: Expr<'source>,
        body: Box<Statement<'source>>,
    },
    Block(Vec<Statement<'source>>),
    Print(Expr<'source>),
    Break,
    Continue,
    Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration<'source> {
    pub var_type: Type,
    pub identifier: &'source str,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program<'source> {
    pub declarations: Vec<Declaration<'source>>,
    pub instructions: Vec<Statement<'source>>,
}
