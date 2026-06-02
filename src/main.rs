pub mod ast;
pub mod sld;
pub mod unify;

#[cfg(test)]
mod tests;

use std::fs;

use lalrpop_util::lalrpop_mod;

use crate::ast::Program;

lalrpop_mod!(pub parser);

fn main() {
    let input = fs::read_to_string("examples/family.pl").expect("couldn't read examples/family.pl");
    let par = parser::ProgramParser::new();

    let program: Program = par.parse(&input).expect("couldn't parse");

    for clause in program {
        println!("{}", clause);
    }
}
