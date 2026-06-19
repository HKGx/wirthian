use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Integer,
    String,
    Boolean,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr<'a> {
    pub kind: ExprKind<'a>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind<'a> {
    Number(i32),
    StringLit(&'a str),
    Identifier(&'a str),
    True,
    False,

    Add(&'a Expr<'a>, &'a Expr<'a>),
    Sub(&'a Expr<'a>, &'a Expr<'a>),
    Mul(&'a Expr<'a>, &'a Expr<'a>),
    Div(&'a Expr<'a>, &'a Expr<'a>),
    Mod(&'a Expr<'a>, &'a Expr<'a>),

    Concatenate(&'a Expr<'a>, &'a Expr<'a>),
    Substring(&'a Expr<'a>, &'a Expr<'a>, &'a Expr<'a>),
    Length(&'a Expr<'a>),
    Position(&'a Expr<'a>, &'a Expr<'a>),

    Not(&'a Expr<'a>),
    And(&'a Expr<'a>, &'a Expr<'a>),
    Or(&'a Expr<'a>, &'a Expr<'a>),
    Eq(&'a Expr<'a>, &'a Expr<'a>),
    Neq(&'a Expr<'a>, &'a Expr<'a>),
    Less(&'a Expr<'a>, &'a Expr<'a>),
    LessEq(&'a Expr<'a>, &'a Expr<'a>),
    Greater(&'a Expr<'a>, &'a Expr<'a>),
    GreaterEq(&'a Expr<'a>, &'a Expr<'a>),

    ReadInt,
    ReadStr,
    ReadBool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement<'a> {
    pub kind: StmtKind<'a>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind<'a> {
    Assign(&'a str, &'a Expr<'a>),
    If {
        condition: &'a Expr<'a>,
        then_branch: &'a Statement<'a>,
        elif_branches: Vec<(&'a Expr<'a>, &'a Statement<'a>)>,
        else_branch: Option<&'a Statement<'a>>,
    },
    For {
        iterator: &'a str,
        from: &'a Expr<'a>,
        to: &'a Expr<'a>,
        body: &'a Statement<'a>,
    },
    Block(Vec<&'a Statement<'a>>),
    Print(&'a Expr<'a>),
    Break,
    Continue,
    Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration<'a> {
    pub var_type: Type,
    pub identifier: &'a str,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program<'a> {
    pub declarations: Vec<Declaration<'a>>,
    pub instructions: Vec<&'a Statement<'a>>,
}
