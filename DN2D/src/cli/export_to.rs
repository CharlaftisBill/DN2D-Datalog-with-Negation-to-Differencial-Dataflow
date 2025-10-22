use std::{fs, fmt, path::PathBuf, str::FromStr};

use serde::Serialize;

#[derive(Debug, Clone)]
pub enum ExportTo {
    Path(PathBuf),
    Print,
    None,
}

impl ExportTo {
    pub fn handle(&self, json_str: String) {
        match self {
            ExportTo::Path(export_path) => {
                fs::write(&export_path, json_str)
                    .unwrap_or_else(|err| {
                        panic!(
                            "Error: Could not write lex tokens json to file '{}': {}",
                            export_path.display(),
                            err
                        )
                    }
                );
            }
            ExportTo::Print => {
                println!("______ Print as Json ______");
                println!("\n{}\n", json_str);
                println!("____________________________");
            },
            ExportTo::None => {}
        }
    }
}

impl FromStr for ExportTo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "print" => Ok(ExportTo::Print),
            "none" => Ok(ExportTo::None),
            path => Ok(ExportTo::Path(PathBuf::from(path))),
        }
    }
}

impl std::fmt::Display for ExportTo{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportTo::Path(p) => write!(f, "{}", p.display()),
            ExportTo::Print => write!(f, "print"),
            ExportTo::None => write!(f, "none"),
        }
    }
}

// helper
pub fn to_json_str<T: Serialize>(value: &T)  -> String {
    serde_json::to_string_pretty(&value).unwrap_or_else(|err| {
        panic!("Error: Failed to serialize to JSON string: {}", err);
    })
} 