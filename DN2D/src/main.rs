use core::fmt;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::process;

mod analisis;
mod ast;
mod cli;
mod lexer;

use crate::analisis::planner::OrderedProgram;
use crate::ast::dataflow::operator::NodeId;
use crate::ast::datalog::rule_or_fact::Fact;
use crate::ast::datalog::Constant;
use crate::ast::datalog::Expression;
use crate::ast::datalog::Parsable;
use crate::ast::datalog::Program;
use crate::ast::datalog::ReadDirective;
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

    diff_adapt(ordered_program);

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

fn diff_adapt(ordered_program: OrderedProgram) {
    let mut input_data_types: HashMap<String, String> = HashMap::new();

    let mut input_session = String::with_capacity(4096);
    let mut input_collections = String::with_capacity(4096);
    let mut fact_inserts = String::with_capacity(4096);

    for input in ordered_program.inputs {
        let input_name = &input.name.0;

        render_input_type_tuples(&input, &mut input_data_types);
        render_input_session(input_name, &mut input_session);
        render_input_collection(input_name, &mut input_collections);
    }

    for fact in ordered_program.facts {
        render_fact_type_tuples(&fact, &mut input_data_types);
        render_fact_inserts(&fact, &mut fact_inserts);
    }

    for (_, input_data_type) in input_data_types {
        println!("\n{}", input_data_type);
    }
    println!("\n{}\n{}\n{}", input_session, input_collections, fact_inserts);
}

// INPUTS:
// type exampleInputData = (String, String);
fn render_input_type_tuples(input: &ReadDirective, input_data_types: &mut HashMap<String, String>) {
    let input_name = &input.name.0;

    if input_data_types.contains_key(input_name) {
        return;
    }

    let input_data_type = &mut String::with_capacity(4096);
    write!(input_data_type, "type {}InputData = (", input_name).unwrap();

    for (index, column) in input.columns.iter().enumerate() {
        if index != 0 {
            input_data_type.push_str(", ");
        }
        write!(input_data_type, "{column}").unwrap();
    }

    input_data_type.push_str(");\n");

    input_data_types.insert(input_name.into(), input_data_type.to_string());
}

// let mut example_as_input = InputSession::<isize, {}InputData, isize>::new();
fn render_input_session(input_name: &String, input_session: &mut String) {
    write!(
        input_session,
        "let mut {}_as_input = InputSession::<isize, {}InputData, isize>::new();\n",
        input_name, input_name
    )
    .unwrap();
}

// let example_as_collection = example_as_input.to_collection(scope);
fn render_input_collection(input_name: &String, input_collections: &mut String) {
    write!(
        input_collections,
        "let {}_as_collection = {}_as_input.to_collection(scope);\n",
        input_name, input_name
    )
    .unwrap();
}

// FACTS:
// type exampleInputData = (String, String);
fn render_fact_type_tuples(input: &Fact, input_data_types: &mut HashMap<String, String>) {
    let fact_name = &input.head.name.0;
    if input_data_types.contains_key(fact_name) {
        return;
    }

    let input_data_type = &mut String::with_capacity(4096);

    write!(input_data_type, "type {}InputData = (", fact_name).unwrap();

    for (index, term) in input.head.terms.iter().enumerate() {
        if index != 0 {
            input_data_type.push_str(", ");
        }

        if let Expression::Constant(constnt) = term {
            write!(input_data_type, "{}", constnt).unwrap();
        }
    }

    input_data_type.push_str(");\n");

    input_data_types.insert(fact_name.into(), input_data_type.to_string());
}

// example_as_input.insert(("a".to_string(), "b".to_string(), ...));
fn render_fact_inserts(input: &Fact, fact_inserts:  &mut String) {
    let fact_name = &input.head.name.0;

    write!(fact_inserts, "{}_as_input.insert((", fact_name).unwrap();

    for (index, term) in input.head.terms.iter().enumerate() {
        if index != 0 {
            fact_inserts.push_str(", ");
        }

        if let Expression::Constant(constnt) = term {
            write!(fact_inserts, "\"{}\".to_string()", constnt.value()).unwrap();
        }
    }

    fact_inserts.push_str("));\n");
}
