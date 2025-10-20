// src/main.rs

use std::env;
use std::process;
use std::fs;

// Declare the modules
mod lexer;
mod ast;

// Bring items into scope
use lexer::Lexer;
use ast::Parser;

use crate::ast::Parsable;
use crate::ast::Program;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }
    let filename = &args[1];

    let source_code = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error: Could not read file '{}': {}", filename, err);
        process::exit(1);
    });

    println!("--- Lexing file: {} ---", filename);
    let lexer = Lexer::new(&source_code);
    let tokens: Vec<_> = match lexer.collect::<Result<_, _>>() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    println!("Lexing successful. Found {} tokens.", tokens.len());

    println!("\n--- Parsing tokens into AST ---");
    let mut parser = Parser::new(&source_code, tokens);
    let program_ast = match Program::parse(&mut parser) {
        Ok(ast) => {
            println!("Parsing successful.");
            ast
        },
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    println!("\n--- Abstract Syntax Tree (JSON) ---");
    let json_output = serde_json::to_string_pretty(&program_ast).unwrap_or_else(|err| {
        eprintln!("Error: Failed to serialize AST to JSON: {}", err);
        process::exit(1);
    });

    println!("{}", json_output);
}