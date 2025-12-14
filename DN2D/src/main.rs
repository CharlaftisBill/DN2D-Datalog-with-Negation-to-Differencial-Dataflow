use std::collections::HashMap;
use std::fs;
use std::process;

mod analisis;
mod ast;
mod cli;
mod lexer;

use crate::analisis::planner::OrderedProgram;
use crate::ast::dataflow::operator::NodeId;
use crate::ast::datalog::Parsable;
use crate::ast::datalog::Program;
use crate::cli::Command;

use crate::ast::dataflow::Program as DdProgram;

use ast::datalog::Parser;
use lexer::Lexer;

use crate::lexer::Token;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Command::new();

    let filename = cli.src_path.to_str().unwrap_or_default().to_string();

    let source_code = fs::read_to_string(filename.clone()).unwrap_or_else(|err| {
        eprintln!("Error: Could not read file '{}': {}", filename, err);
        process::exit(1);
    });

    let tokens = lex(&source_code);
    cli.lex_as_json.handle(cli::export_to::to_json_str(&tokens));

    let program_skeleton = parse(&source_code, tokens)?;
    cli.ast_as_json
        .handle(cli::export_to::to_json_str(&program_skeleton));

    let ordered_program = validate_and_plan(&source_code, program_skeleton)?;
    cli.ordered_ast_as_json
        .handle(cli::export_to::to_json_str(&ordered_program));

    Ok(())
}

fn lex(source_code: &String) -> Vec<Token> {
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

fn parse(source_code: &String, tokens: Vec<Token>) -> Result<Program, Box<dyn std::error::Error>> {
    let mut parser = Parser::new(&source_code, tokens);
    let program_ast = Program::parse(&mut parser)?;

    Ok(program_ast)
}

fn validate_and_plan(
    source_code: &String,
    program_skeleton: Program,
) -> Result<OrderedProgram, Box<dyn std::error::Error>> {
    let validator = analisis::Validator::new(&program_skeleton);
    let planer = validator.validate(&source_code)?;
    Ok(planer.plan())
}

fn DiffAdapt(ordered_program: OrderedProgram) -> DdProgram {
    let mut program = DdProgram::new();
    let mut cache: HashMap<String, NodeId> = HashMap::new();

    // for input in ordered_program.inputs {
    //     cache.insert(
    //         input.name.0,
    //         program.add_input(input.name.0)
    //     );
    // }

    for fact in ordered_program.facts {
        cache.entry(fact.head.name.0).or_insert(
            program.add_input(fact.head.name.0)
        );
    }

    for strtm in ordered_program.strata {
        if strtm.is_recursive {
            program.add_iterate(input, step_root);
        }
    }

    program
}
