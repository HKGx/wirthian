pub mod ast;
pub mod checker;
pub mod interp;
pub mod lexer;

#[cfg(feature = "cli")]
pub mod highlighter;

#[cfg(feature = "wasm")]
pub mod wasm;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar);
