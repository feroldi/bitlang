#![feature(hash_raw_entry, hasher_prefixfree_extras)]

use crate::driver::compile;

mod ast;
mod codegen;
mod compiler_context;
mod driver;
mod interner;
mod parser;
mod scanner;

#[cfg(test)]
mod tests;

fn main() {
    let _ = compile("main :: () {}");
}
