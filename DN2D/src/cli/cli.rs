use clap::Parser;
use std::path::PathBuf;

use crate::cli::export_to::ExportTo;

#[derive(Parser, Debug)]
#[command(name = "dn2d")]
#[command(about = "Datalog with Negation to Differential Dataflow", long_about = None)]
pub struct Command {
    #[arg(long, default_value = "none")]
    pub lex_as_json: ExportTo,
    
    #[arg(long, default_value = "none")]
    pub ast_as_json: ExportTo,

    #[arg()]
    pub src_path: PathBuf,
}

impl Command {
    pub fn new() -> Self{
        Command::parse()
    }
}
