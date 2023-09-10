#![feature(hash_raw_entry, hasher_prefixfree_extras)]

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
    println!("Hello, world!");
}
