mod code_checker;
mod context;
mod function;
mod instruction;
mod optimizer;
mod producer;
mod variable;

use std::collections::{HashMap, HashSet};
use std::fs;

use anyhow::{anyhow, Context, Result};
use pest::Parser;
extern crate pest;
use pest::iterators::Pair;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

use code_checker::{transform_goto, Outputpointer};
use function::{produce_mapping, replace_function};
use instruction::Instruction;
use optimizer::optimize_consecutive;
use producer::produce_string;

lazy_static! {
    static ref STD_FUNCTION: Vec<&'static str> = vec!["SetConst", "Goto", "Add", "Sub", "Print"];
}

fn main() {
    /* let name = std::env::args()
    .nth(1)
    .expect("you need to specify a file to compile"); */
    let name = "test.bfil";
    let file = fs::read_to_string(name).expect("cannot read file");
    let (ist_res, function_name) = Instruction::parse(file); //.expect("failed to parse programs")
    let mut ist = ist_res.expect("failed to parse programs");
    dbg!(&ist);
    let (ist, mapping) = produce_mapping(ist, function_name).expect("cannot produce mapping");
    let mut set = HashSet::new();
    let ist = replace_function(&ist, &mapping, &mut set).expect("cannot replace function");

    let (mut res, _) = transform_goto(ist, Outputpointer::default());

    let brainfuck_code = optimize_consecutive(res.expect("failed to transform goto"))
        .into_iter()
        .map(|ist_inner| produce_string(ist_inner))
        .collect::<String>();

    dbg!(brainfuck_code);
    println!("Hello, world!");
}
