pub mod ast;
pub mod checker;
pub mod highlighter;
pub mod interp;
pub mod lexer;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar);
