use differential_dataflow::operators::*;
use differential_dataflow::input::InputSession;
use differential_dataflow::operators::arrange::ArrangeByKey;

fn main() {
    timely::execute::execute_from_args(std::env::args(), |worker| {

        let mut edges_in = InputSession::<i32, (String, String), isize>::new();
        let mut probe = timely::dataflow::ProbeHandle::new();

        worker.dataflow(|scope| {
            let edges = edges_in.to_collection(scope);

            let arranged_edges = edges.arrange_by_key();

            let reachable = edges.iterate(|frontier| {
                let edges_inner = arranged_edges.enter(&frontier.scope());
                let edges_base = edges.enter(&frontier.scope());

                let new_paths = frontier
                    .map(|(x, y)| (y, x))
                    .join_core(&edges_inner, |_y, x, z| Some((x.clone(), z.clone())));

                new_paths.concat(&edges_base).distinct()
            });

            let reach_counts = reachable
                .map(|(node, _neighbor)| node)
                .count();

            reach_counts.inspect(|x| println!("ReachCount: {:?}", x));
            reach_counts.probe_with(&mut probe);
        });

        // The following code is not reflected in .dl file as the following operation should be happen
        // into the input `.csv` file  
        println!("\n--- Transaction 1: Loading Graph Data ---");
        
        // Graph: 1 -> 2 -> 3 -> 4
        // Node 1 can reach: 2, 3, 4 (Count = 3)
        // Node 2 can reach: 3, 4    (Count = 2)
        // Node 3 can reach: 4       (Count = 1)
        // Node 4 can reach: (none)  (Count = 0 - implicitly not present)
        
        edges_in.insert(("1".to_string(), "2".to_string()));
        edges_in.insert(("2".to_string(), "3".to_string()));
        edges_in.insert(("3".to_string(), "4".to_string()));

        edges_in.advance_to(1);
        
        edges_in.flush();

        while probe.less_than(edges_in.time()) {
            worker.step();
        }

        println!("\n--- Transaction 2: Adding a new edge (1 -> 5) ---");
        edges_in.insert(("1".to_string(), "5".to_string()));
        
        edges_in.advance_to(2);
        edges_in.flush();
        while probe.less_than(edges_in.time()) {
            worker.step();
        }

        println!("\n--- Transaction 3: Removing the bridge (2 -> 3) ---");
        edges_in.remove(("2".to_string(), "3".to_string()));

        edges_in.advance_to(3);
        edges_in.flush();
        while probe.less_than(edges_in.time()) {
            worker.step();
        }

    }).unwrap();
}