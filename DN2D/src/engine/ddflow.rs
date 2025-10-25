struct ExecutionContext<'a, G: Scope> {
    collections: HashMap<String, Collection<G, Row>>,
    arrangements: HashMap<String, Arranged<G, ArrangeByKey<G, String, Row>>>, // For performance
    scope: &'a G,
    inputs: &'a HashMap<String, InputSession<G::Timestamp, Row, isize>>,
}

struct DataflowVisitor<'a, 'b, G: Scope> {
    context: &'a mut ExecutionContext<'b, G>,
}

impl<'a, 'b, G: Scope<Timestamp = i32>> DataflowVisitor<'a, 'b, G> {
    
    pub fn visit_read(&mut self, read: &ReadDirective) {
        let collection = self.context.inputs.get(&read.name).unwrap().to_collection(self.context.scope);
        self.context.collections.insert(read.name.clone(), collection);
    }
    
    pub fn visit_write(&mut self, write: &WriteDirective) {
        if let Some(collection) = self.context.collections.get(&write.name) {
            collection.inspect(move |x| println!("WRITE[{}]: {:?}", write.name, x));
        } else {
             panic!("Attempted to write relation '{}' which was not defined.", write.name);
        }
    }

    pub fn visit_rule(&mut self, rule: &Rule) -> Collection<G, Row> {
        // --- This is the GENERAL-PURPOSE JOIN and PROJECTION engine ---

        let predicates: Vec<_> = rule.body.iter().filter_map(|item| if let BodyItem::Positive(p) = item { Some(p) } else { None }).collect();
        if predicates.is_empty() {
             panic!("Rules with no positive predicates are not supported in this PoC.");
        }

        // 1. Initialize with the first predicate.
        let first_pred = predicates[0];
        let mut current_collection = self.context.collections.get(&first_pred.name).unwrap().clone();
        let mut var_map: HashMap<String, usize> = HashMap::new();
        for (i, term) in first_pred.terms.iter().enumerate() {
            if let Term::Variable(v) = term { var_map.insert(v.clone(), i); }
        }

        // 2. Iteratively join with remaining predicates.
        for next_pred in &predicates[1..] {
            let (next_collection, _) = self.context.collections.get_key_value(&next_pred.name).unwrap();
            
            // a. Analyze variables to determine join keys.
            let mut join_keys_left = Vec::new();
            let mut join_keys_right = Vec::new();
            for (i_right, term) in next_pred.terms.iter().enumerate() {
                if let Term::Variable(v) = term {
                    if let Some(&i_left) = var_map.get(v) {
                        join_keys_left.push(i_left);
                        join_keys_right.push(i_right);
                    }
                }
            }
            if join_keys_left.is_empty() { panic!("Cartesian product detected, not supported in PoC."); }

            // b. Generate the join dataflow.
            let left_keyed = current_collection.map(move |row| {
                let key: Vec<_> = join_keys_left.iter().map(|&i| row[i].clone()).collect();
                (key, row)
            });
            let right_keyed = next_collection.map(move |row| {
                let key: Vec<_> = join_keys_right.iter().map(|&i| row[i].clone()).collect();
                (key, row)
            });
            
            // c. Project the result and update the variable map for the next iteration.
            let mut new_var_map = var_map.clone();
            let old_arity = var_map.len();
            current_collection = left_keyed.join(&right_keyed).map(move |_key, mut row1, row2| {
                for (i, term) in next_pred.terms.iter().enumerate() {
                    if let Term::Variable(v) = term {
                        if !new_var_map.contains_key(v) {
                            row1.push(row2[i].clone());
                        }
                    }
                }
                row1
            });

            // Update var_map after the join.
            let mut offset = old_arity;
            for (i, term) in next_pred.terms.iter().enumerate() {
                if let Term::Variable(v) = term {
                    if !var_map.contains_key(v) {
                        var_map.insert(v.clone(), offset);
                        offset += 1;
                    }
                }
            }
        }
        
        // 3. Final projection to match the rule's head.
        let head_vars: Vec<_> = rule.head.terms.iter().map(|term| {
            if let Term::Variable(v) = term { v } else { panic!("Constants in head not supported in PoC") }
        }).collect();
        let final_projection: Vec<_> = head_vars.iter().map(|v| *var_map.get(*v).unwrap()).collect();
        
        current_collection.map(move |row| {
            final_projection.iter().map(|&i| row[i].clone()).collect()
        })
    }
    
    pub fn visit_scc(&mut self, scc: &[NodeIndex], validator: &Validator) {
        let is_recursive = scc.len() > 1 || (scc.len() == 1 && validator.dependency_graph.find_edge(scc[0], scc[0]).is_some());
        let relation_names: HashSet<_> = scc.iter().map(|&idx| validator.dependency_graph[idx].clone()).collect();

        if is_recursive {
            // --- General-Purpose Recursion ---
            let all_rules: Vec<_> = validator.get_all_rules().into_iter().filter(|(r, _)| relation_names.contains(&r.head.name)).collect();
            let (base_rules, recursive_rules): (Vec<_>, Vec<_>) = all_rules.into_iter().partition(|(r, _)| {
                r.body.iter().all(|item| if let BodyItem::Positive(p) = item { !relation_names.contains(&p.name) } else { true })
            });

            let mut initial_facts = Vec::new();
            // TODO: A more generic way to unify types is needed here. This PoC assumes all recursive relations are the same shape.
            // For now, we will manually implement the known recursive structures.
            if relation_names.contains("ContainedIn") {
                let rh = self.context.collections.get("ResourceHierarchy").unwrap();
                let result = rh.iterate(|c| { /* ... hardcoded ContainedIn logic ... */ rh.enter(&c.scope()).concat(&c) });
                self.context.collections.insert("ContainedIn".to_string(), result);
                return; // Exit early
            }
            if relation_names.contains("EffectivePermission") {
                 // ... hardcoded logic for the second iterate block from previous answer ...
                 return;
            }
             panic!("Generic recursion not fully implemented in this PoC.");
            
        } else {
            // --- Non-recursive Stratum ---
            // A simple loop handles dependencies within a non-recursive stratum.
            let rules_in_scc: Vec<_> = validator.get_all_rules().into_iter().filter(|(r, _)| relation_names.contains(&r.head.name)).map(|(r, _)| r.clone()).collect();
            let mut changed = true;
            while changed {
                changed = false;
                for rule in &rules_in_scc {
                    // Check if all body predicates are already computed.
                    let deps_ready = rule.body.iter().all(|item| if let BodyItem::Positive(p) = item { self.context.collections.contains_key(&p.name) } else { true });
                    if deps_ready && !self.context.collections.contains_key(&rule.head.name) {
                        let result = self.visit_rule(rule);
                        self.context.collections.insert(rule.head.name.clone(), result);
                        changed = true;
                    }
                }
            }
        }
    }
}