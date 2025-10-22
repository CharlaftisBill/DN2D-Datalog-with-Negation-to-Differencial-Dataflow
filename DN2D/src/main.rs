use std::process;
use std::fs;

// Declare the modules
mod ast;
mod cli;
mod lexer;

// Bring items into scope
use ast::Parser;
use lexer::Lexer;
use crate::cli::Command;

use crate::ast::Parsable;
use crate::ast::Program;
use crate::lexer::Token;

fn main() {
    let cli = Command::new();

    let filename = cli.src_path
        .to_str()
        .unwrap_or_default()
        .to_string();

    let source_code = fs::read_to_string(filename.clone()).unwrap_or_else(|err| {
        eprintln!("Error: Could not read file '{}': {}", filename, err);
        process::exit(1);
    });

    let tokens = lex(filename, &source_code);
    cli.lex_as_json.handle(cli::export_to::to_json_str(&tokens));

    let mut parser = Parser::new(&source_code, tokens);
    let program_ast = match Program::parse(&mut parser) {
        Ok(ast) => ast,
        Err(e) => panic!("{}", e)
    };
    cli.ast_as_json.handle(cli::export_to::to_json_str(&program_ast));
}

fn lex(filename :String, source_code: &String) -> Vec<Token>{

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

    tokens
}
