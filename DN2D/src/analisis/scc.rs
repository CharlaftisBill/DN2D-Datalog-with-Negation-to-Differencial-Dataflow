use std::collections::HashMap;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::algo::{tarjan_scc, toposort};

use crate::ast::{RuleSpan, rule_or_fact::Rule, Literal, Program, RuleOrFact, Statement};

struct RuleInfo<'a> {
    rule: &'a Rule,
    in_iterate_block: bool,
}

struct RuleDependencies {
    head_name: String,
    body_names: Vec<String>,
}

pub struct ValidationError<'a> {
    pub error_message : String,
    pub span          : &'a RuleSpan,
}

pub struct Validator<'a> {
    ast: &'a Program,
    dependency_graph: Graph<String, ()>,
    symbol_table: HashMap<String, NodeIndex>,
}

impl<'a> Validator<'a> {
    pub fn new(ast: &'a Program) -> Self {
        let mut state = Validator {
            ast,
            dependency_graph: Graph::new(),
            symbol_table: HashMap::new(),
        };
        state.build_dependency_graph();
        state
    }

    pub fn validate_plan(&self) -> Result<Vec<Vec<NodeIndex>>, Vec<ValidationError<'a>>> {
        let sccs = tarjan_scc(&self.dependency_graph);
        let mut errors: Vec<ValidationError<'a>> = Vec::new();

        for scc in &sccs {
            let is_recursive = scc.len() > 1 || 
                (scc.len() == 1 && self.dependency_graph.find_edge(scc[0], scc[0]).is_some());

            if is_recursive {
                self.validate_recursive_scc(&scc, &mut errors);
            }
        }
        
        if !errors.is_empty() {
            return Err(errors);
        }

        let mut node_to_scc_id = HashMap::new();
        for (i, scc) in sccs.iter().enumerate() {
            for &node in scc {
                node_to_scc_id.insert(node, i);
            }
        }

        let mut scc_graph = Graph::<usize, ()>::new();
        let scc_nodes: Vec<_> = (0..sccs.len())
            .map(|i| scc_graph.add_node(i))
            .collect();

        for edge in self.dependency_graph.raw_edges() {
            let source_scc_id = *node_to_scc_id.get(&edge.source()).unwrap();
            let target_scc_id = *node_to_scc_id.get(&edge.target()).unwrap();

            if source_scc_id != target_scc_id {
                scc_graph.add_edge(scc_nodes[source_scc_id], scc_nodes[target_scc_id], ());
            }
        }

        let sorted_scc_indices = toposort(&scc_graph, None)
            .unwrap();

        let execution_plan: Vec<Vec<NodeIndex>> = sorted_scc_indices
            .into_iter()
            .rev() 
            .map(|scc_node_index| {
                let scc_id = scc_graph[scc_node_index];
                sccs[scc_id].clone()
            })
            .collect();

        Ok(execution_plan)
    }

    fn validate_recursive_scc(&self, scc: &[NodeIndex], errors: &mut Vec<ValidationError<'a>>) {

        let relation_names: Vec<String> = scc.iter()
            .map(|&idx| self.dependency_graph[idx].clone())
            .collect();        
        
        let all_rules = self.find_all_rules();

        for rule_info in all_rules {
            if relation_names.contains(&rule_info.rule.head.name.0) && !rule_info.in_iterate_block {

                let error_message = format!(
                    "The rule defining '{}' is part of a recursive definition. Should be in a '.iterate' block.",
                    rule_info.rule.head.name.0,
                );

                
                let err = ValidationError{
                    error_message,
                    span: &rule_info.rule.span
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
                    rules_list.push(RuleInfo { rule, in_iterate_block: false });
                }
                Statement::Iterate(block) => {
                    
                    for item in &block.rules {
                        if let RuleOrFact::Rule(rule) = item {
                            rules_list.push(RuleInfo { rule, in_iterate_block: true });
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
                self.dependency_graph.add_edge(body_node_idx, head_node_idx, ());
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
        
        let body_names = rule.body.iter().filter_map(|literal| {
            if let Literal::Positive(p) = literal {
                Some(p.name.0.clone())
            } else {
                None
            }
        }).collect();

        RuleDependencies {
            head_name,
            body_names,
        }
}
}