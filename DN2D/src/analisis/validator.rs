use petgraph::algo::tarjan_scc;
use petgraph::graph::{Graph, NodeIndex};
use std::collections::HashMap;

use crate::analisis::Planner;
use crate::ast::{rule_or_fact::Rule, Literal, Program, RuleOrFact, RuleSpan, Statement};

struct RuleInfo<'a> {
    rule: &'a Rule,
    in_iterate_block: bool,
}

struct RuleDependencies {
    head_name: String,
    body_names: Vec<String>,
}

pub struct ValidationError<'a> {
    pub error_message: String,
    pub span: &'a RuleSpan,
}

pub struct Validator<'a> {
    pub ast: &'a Program,
    pub dependency_graph: Graph<String, ()>,
    symbol_table: HashMap<String, NodeIndex>,
}

impl<'a> Validator<'a> {
    pub fn new(ast: &'a Program) -> Self {
        let mut validator = Validator {
            ast,
            dependency_graph: Graph::new(),
            symbol_table: HashMap::new(),
        };
        validator.build_dependency_graph();
        validator
    }

    pub fn validate(&'_ self, source_code: &String) -> Result<Planner<'_>, Box<dyn std::error::Error>> {
        let sccs: Vec<Vec<NodeIndex>> = tarjan_scc(&self.dependency_graph);
        let mut errors: Vec<ValidationError<'a>> = Vec::new();

        for scc in &sccs {
            if scc.len() > 1
                || (scc.len() == 1 && self.dependency_graph.find_edge(scc[0], scc[0]).is_some()){
                self.validate_recursive_scc(&scc, &mut errors);
            }
        }

        if !errors.is_empty() {
            self.report_validation_errors(source_code, errors);
            return Err("Terminating".into())
        }

        Ok(Planner::new(&self, sccs))
    }

    fn report_validation_errors(&self, source_code: &String, errors: Vec<ValidationError<'_>>) {
        println!("\n\x1b[31mValidation ERROR(s):\x1b[0m");

        let padding = source_code.lines().count().to_string().len();
        errors.iter().for_each(|e| {
            for index in e.span.line_start - 1..e.span.line_end {
                print!(
                    "{:^width$}┃ {}",
                    index + 1,
                    source_code.lines().nth(index).unwrap(),
                    width = padding
                );

                if index == e.span.line_end - 1 {
                    println!(" \x1b[31m{}\x1b[0m", e.error_message);
                    println!("{:^width$}┃", "⋮", width = padding,);
                } else {
                    println!();
                }
            }
        });
    }

    fn validate_recursive_scc(&self, scc: &[NodeIndex], errors: &mut Vec<ValidationError<'a>>) {
        let relation_names: Vec<String> = scc
            .iter()
            .map(|&idx| self.dependency_graph[idx].clone())
            .collect();

        let all_rules = self.find_all_rules();

        for rule_info in all_rules {
            if relation_names.contains(&rule_info.rule.head.name.0) && !rule_info.in_iterate_block {
                let error_message = format!(
                    "The rule defining '{}' is part of a recursive definition. Should be in a '.iterate' block.",
                    rule_info.rule.head.name.0,
                );

                let err = ValidationError {
                    error_message,
                    span: &rule_info.rule.span,
                };

                errors.push(err);
            }
        }
    }

    fn find_all_rules(&self) -> Vec<RuleInfo<'a>> {
        let mut rules_list = Vec::new();

        for stmt in &self.ast.statements {
            match stmt {
                Statement::Rule(rule) => {
                    rules_list.push(RuleInfo {
                        rule,
                        in_iterate_block: false,
                    });
                }
                Statement::Iterate(block) => {
                    for item in &block.rules {
                        if let RuleOrFact::Rule(rule) = item {
                            rules_list.push(RuleInfo {
                                rule,
                                in_iterate_block: true,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        rules_list
    }

    fn build_dependency_graph(&mut self) {
        for deps in self.collect_rule_dependencies() {
            let head_node_idx = self.get_or_create_node(&deps.head_name);

            for body_name in deps.body_names {
                let body_node_idx = self.get_or_create_node(&body_name);
                self.dependency_graph
                    .add_edge(body_node_idx, head_node_idx, ());
            }
        }
    }

    fn get_or_create_node(&mut self, name: &str) -> NodeIndex {
        if let Some(idx) = self.symbol_table.get(name) {
            *idx
        } else {
            let idx = self.dependency_graph.add_node(name.to_string());
            self.symbol_table.insert(name.to_string(), idx);
            idx
        }
    }

    fn collect_rule_dependencies(&self) -> Vec<RuleDependencies> {
        let mut all_deps = Vec::new();

        for stmt in &self.ast.statements {
            match stmt {
                Statement::Rule(rule) => {
                    all_deps.push(self.extract_deps_from_rule(rule));
                }
                Statement::Iterate(block) => {
                    for in_block_rule in &block.rules {
                        if let RuleOrFact::Rule(rule) = in_block_rule {
                            all_deps.push(self.extract_deps_from_rule(rule));
                        }
                    }
                }
                _ => {}
            }
        }
        all_deps
    }

    fn extract_deps_from_rule(&self, rule: &Rule) -> RuleDependencies {
        let head_name = rule.head.name.0.clone();

        let body_names = rule
            .body
            .iter()
            .filter_map(|literal| {
                if let Literal::Positive(p) = literal {
                    Some(p.name.0.clone())
                } else {
                    None
                }
            })
            .collect();

        RuleDependencies {
            head_name,
            body_names,
        }
    }
}
