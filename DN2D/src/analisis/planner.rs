use petgraph::algo::toposort;
use petgraph::graph::{Graph, NodeIndex};
use std::collections::HashMap;

use crate::analisis::Validator;
use crate::ast::rule_or_fact::Rule;
use crate::ast::{Program, ReadDirective, RuleOrFact, Statement, WriteDirective};

pub struct Planner<'a> {
    valid_program: &'a Validator<'a>,
    sccs: Vec<Vec<NodeIndex>>,
}

pub struct Stratum {
    pub is_recursive: bool,
    pub relation_names: Vec<String>,
    pub rules: Vec<Rule>,
}

pub struct OrderedProgram {
    pub inputs: Vec<ReadDirective>,
    pub strata: Vec<Stratum>,
    pub outputs: Vec<WriteDirective>,
}

type ExecutionPlan = Vec<Vec<NodeIndex>>;

impl<'a> Planner<'a> {
    pub fn new(valid_program: &'a Validator<'a>, sccs: Vec<Vec<NodeIndex>>) -> Self {
        Planner {
            valid_program,
            sccs,
        }
    }

    pub fn plan(&self) -> OrderedProgram {
        let execution_plan: ExecutionPlan = self.make_execution_plan();

        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        
        let mut rules_as_map: HashMap<String, Vec<Rule>> = HashMap::new();
        for stmt in &self.valid_program.ast.statements {
            match stmt {
                Statement::Read(r) => inputs.push(r.clone()),
                Statement::Write(w) => outputs.push(w.clone()),
                Statement::Rule(r) => {
                    rules_as_map.entry(r.head.name.0.clone())
                        .or_default().push(r.clone());
                }
                Statement::Iterate(block) => {
                    for wrapped in &block.rules {
                        if let RuleOrFact::Rule(r) = wrapped {
                            rules_as_map.entry(r.head.name.0.clone())
                                .or_default().push(r.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        let strata = self.make_strata(execution_plan, rules_as_map);
        
        OrderedProgram {
            inputs,
            strata,
            outputs,
        }
    }

    fn make_execution_plan(&self) -> ExecutionPlan {
        let mut node_to_scc_id = HashMap::new();
        for (i, scc) in self.sccs.iter().enumerate() {
            for &node in scc {
                node_to_scc_id.insert(node, i);
            }
        }

        let mut scc_graph = Graph::<usize, ()>::new();
        let scc_nodes: Vec<_> = (0..self.sccs.len())
            .map(|i| scc_graph.add_node(i))
            .collect();

        for edge in self.valid_program.dependency_graph.raw_edges() {
            let source_scc_id = *node_to_scc_id.get(&edge.source()).unwrap();
            let target_scc_id = *node_to_scc_id.get(&edge.target()).unwrap();

            if source_scc_id != target_scc_id {
                scc_graph.add_edge(scc_nodes[source_scc_id], scc_nodes[target_scc_id], ());
            }
        }

        let sorted_scc_indices = toposort(&scc_graph, None).unwrap();

        sorted_scc_indices
            .into_iter()
            .map(|scc_node_index| {
                let scc_id = scc_graph[scc_node_index];
                self.sccs[scc_id].clone()
            })
            .collect()

    }

    fn make_strata(&self, execution_plan: ExecutionPlan, mut rules_as_map: HashMap<String, Vec<Rule>>) -> Vec<Stratum> {
        let mut strata = Vec::new();

        for scc in execution_plan {
            let mut stratum_rules = Vec::new();
            let mut stratum_names = Vec::new();

            let is_recursive = scc.len() > 1 || 
                (scc.len() == 1 && self.valid_program.dependency_graph.find_edge(scc[0], scc[0]).is_some());

            for node_index in scc {
                let relation_name = &self.valid_program.dependency_graph[node_index];
                stratum_names.push(relation_name.clone());

                if let Some(rules) = rules_as_map.remove(relation_name) {
                    stratum_rules.extend(rules);
                }
            }

            if !stratum_rules.is_empty() {
                strata.push(Stratum {
                    is_recursive,
                    relation_names: stratum_names,
                    rules: stratum_rules,
                });
            }
        }

        strata
    }

}
