// #![no_std]
#[macro_use]
extern crate alloc;

mod combinator;
pub use combinator::{builtin, foreign, Combinator, Nil, B, C, I, K, S};

mod lambda;
pub use lambda::Lambda;

mod compile;
pub use compile::{CompilerError, Golang, Target};

pub(crate) mod ast;

pub(crate) mod reduce;
pub use reduce::{Reduce, ReductionError};

mod parse;
pub use parse::{
    expression::parse_expression,
    statements::{
        parse_ast, parse_constructor_declaration, parse_data_declaration,
        parse_function_declaration, parse_type_declaration,
    },
};

pub fn compiler_error(message: impl ToString) -> ! {
    eprintln!("error: {}", message.to_string());
    std::process::exit(1);
}
