pub mod ast;
pub mod parser;
pub mod sld;
pub mod unify;

#[cfg(test)]
mod tests;

use std::fs;

fn main() {
    let input = fs::read_to_string("examples/family.pl").expect("couldn't read examples/family.pl");
    let program = parser::parse(&input).expect("couldn't parse");

    for clause in program {
        println!("{}", clause);
    }
}
