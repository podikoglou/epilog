pub mod ast;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub parser);

fn main() {
    println!("Hello, world!");
}
