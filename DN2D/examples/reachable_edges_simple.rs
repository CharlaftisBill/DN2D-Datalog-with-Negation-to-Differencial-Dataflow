use differential_dataflow::input::InputSession;
use differential_dataflow::operators::*;

// input types
type EdgeInputData = (String, String);

fn main() {
    timely::execute::execute_from_args(std::env::args(), |worker| {
        
        // 1. Create Input (Corresponding to Facts + .read)
        let mut edge_as_input = InputSession::<u32, EdgeInputData, isize>::new();
        let mut probe = timely::dataflow::ProbeHandle::new();

        worker.dataflow(|scope| {
            // Convert input to a Collection
            let edge_as_collection: differential_dataflow::Collection<timely::dataflow::scopes::Child<'_, timely::worker::Worker<timely::communication::Allocator>, u32>, Vec<((String, String), u32, isize)>> = edge_as_input.to_collection(scope);

            // --- The .iterate block ---
            // Reachable(x, z) :- Reachable(x, y), Edge(y, z).
            let reachable = edge_as_collection.iterate(|reachable_in_iterator| {
                
                // We need 'Edge' inside the loop too.
                let edge_in_iterator = edge_as_collection.enter(&reachable_in_iterator.scope());

                // PREPARE FOR JOIN:
                // To join Reachable(x, y) and Edge(y, z), we must key them both by 'y'.
                
                // 1. Reachable(x, y) -> Key: y, Value: x
                let reachable_in_iterator_by_y = reachable_in_iterator.map(|(x, y)| (y, x));
               
                // EXECUTE JOIN:
                // Result looks like: (Key, Val1, Val2) -> (y, x, z)
                let reachable_in_iterator_by_y_joined_edge_in_iterator = reachable_in_iterator_by_y.join(&edge_in_iterator)
                    // PROJECT: We want Reachable(x, z)
                    .map(|(_y, (x, z))| (x, z));

                // MERGE: Add the new paths to the existing edges (Base Case)
                reachable_in_iterator_by_y_joined_edge_in_iterator.concat(&edge_in_iterator).distinct()
            });

            // --- The Aggregation Rule ---
            // ReachCount(node, count) :- Reachable(node, neighbor).
            let reach_counts = reachable
                // We want to count neighbors per node.
                // Map to just the 'node', ignoring the specific 'neighbor'.
                .map(|(node, _neighbor)| node)
                // The .count() operator counts how many times each 'node' appears.
                .count();

            // --- The .write directive ---
            reach_counts.inspect(|x| println!("ReachCount: {:?}", x));
            reach_counts.probe_with(&mut probe);
        });

        // --- Driver Logic (Loading the Facts) ---
        println!("\n--- Loading Facts ---");
        edge_as_input.insert(("a".to_string(), "b".to_string()));
        edge_as_input.insert(("b".to_string(), "c".to_string()));
        edge_as_input.insert(("c".to_string(), "a".to_string()));

        // Run the computation
        edge_as_input.advance_to(1);
        edge_as_input.flush();
        while probe.less_than(edge_as_input.time()) {
            worker.step();
        }

    }).unwrap();
}