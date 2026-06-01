pub mod ast;

#[cfg(test)]
mod tests;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub parser);

fn main() {}
