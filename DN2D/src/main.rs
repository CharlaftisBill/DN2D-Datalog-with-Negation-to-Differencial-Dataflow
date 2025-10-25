use std::fs;
use std::process;

// Declare the modules
mod ast;
mod cli;
mod lexer;
mod analisis;

// Bring items into scope
use ast::Parser;
use lexer::Lexer;
use crate::analisis::scc::ValidationError;
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

    let tokens = lex(&source_code);
    cli.lex_as_json.handle(cli::export_to::to_json_str(&tokens));

    let mut parser = Parser::new(&source_code, tokens);
    let program_ast = match Program::parse(&mut parser) {
        Ok(ast) => ast,
        Err(e) => panic!("{}", e)
    };
    cli.ast_as_json.handle(cli::export_to::to_json_str(&program_ast));

    let validator = analisis::Validator::new(&program_ast);
    let execution_plan = match validator.validate() {
        Ok(plan) => {
            println!("\n\x1b[32mValidation PASSED\x1b[0m");
            plan
        }
        Err(errors) => {
            validation_error(source_code, errors);
            return;
        }
    };

}

fn validation_error(source_code: String, errors: Vec<ValidationError<'_>>) {
    println!("\n\x1b[31mValidation ERROR(s):\x1b[0m");

    let padding =source_code.lines().count().to_string().len();
    errors.iter().for_each(
        |e|{
            for index in e.span.line_start-1..e.span.line_end {
                print!("{:^width$}┃ {}",
                    index + 1,
                    source_code.lines().nth(index).unwrap(),
                    width = padding
                );

                if index == e.span.line_end -1 {
                    println!(" \x1b[31m{}\x1b[0m", e.error_message);
                    println!("{:^width$}┃", 
                        "⋮",
                        width = padding,
                    );
                }else{
                    println!();
                }
            }
        }
    );
}

fn lex(source_code: &String) -> Vec<Token>{
    let lexer = Lexer::new(&source_code);
    let tokens: Vec<_> = match lexer.collect::<Result<_, _>>() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    tokens
}
