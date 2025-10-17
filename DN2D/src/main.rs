// src/main.rs

use std::env;
use std::process;
use std::fs;

// Declare the modules
mod lexer;
mod parser;

// Bring items into scope
use lexer::Lexer;
use parser::Parser;

fn main() {
    // 1. Parse Command-Line Arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }
    let filename = &args[1];

    // 2. Read the source file
    let source_code = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error: Could not read file '{}': {}", filename, err);
        process::exit(1);
    });

    // 3. Lexing: Turn source code into tokens
    println!("--- 1. Lexing file: {} ---", filename);
    let lexer = Lexer::new(&source_code);
    let tokens: Vec<_> = match lexer.collect::<Result<_, _>>() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    println!("Lexing successful. Found {} tokens.", tokens.len());

    // 4. Parsing: Turn tokens into an AST
    println!("\n--- 2. Parsing tokens into AST ---");
    let mut parser = Parser::new(&source_code, tokens);
    let program_ast = match parser.parse_program() {
        Ok(ast) => {
            println!("Parsing successful.");
            ast
        },
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    // 5. Output: Serialize AST to JSON and print
    println!("\n--- 3. Abstract Syntax Tree (JSON) ---");
    let json_output = serde_json::to_string_pretty(&program_ast).unwrap_or_else(|err| {
        eprintln!("Error: Failed to serialize AST to JSON: {}", err);
        process::exit(1);
    });

    println!("{}", json_output);
}