### Tutorial 1: The "Hello, World!" - Getting Data In and Out

The simplest dataflow takes an input, does nothing to it, and shows the output. This teaches us the basic boilerplate.

**Concepts:**
*   `timely::execute...`: The boilerplate to start a dataflow computation.
*   `InputSession`: The handle you use to push data into the system.
*   `.to_collection()`: Converts an input handle into a ddflow `Collection`.
*   `.inspect()`: The simplest way to see the output stream of changes.
*   `.advance_to()` & `.flush()`: How you advance the logical time and push data into the computation.

**Code (`tutorial_1.rs`):**
```rust
use timely::dataflow::operators::{Input, Inspect};
use differential_dataflow::input::InputSession;

fn main() {
    timely::execute::execute_from_args(std::env::args(), |worker| {
        // 1. Create a handle to push data into the dataflow.
        let mut input = InputSession::new();

        worker.dataflow(|scope| {
            // 2. Turn the input handle into a live `Collection`.
            let collection = input.to_collection(scope);

            // 3. Attach an inspector to print every change that flows out.
            collection.inspect(|change| println!("Output: {:?}", change));
        });

        // 4. Drive the computation.
        println!("--- Time 1: Inserting two names ---");
        input.insert("alice".to_string());
        input.insert("bob".to_string());
        input.advance_to(1); // Advance the logical clock to 1
        input.flush();        // Push all buffered changes into the dataflow
        worker.step_while(|| input.time().less_than(&1)); // Run computation until time 1 is done

        println!("\n--- Time 2: Inserting one name, removing another ---");
        input.insert("charlie".to_string());
        input.remove("alice".to_string()); // `remove` sends a -1 diff
        input.advance_to(2);
        input.flush();
        worker.step_while(|| input.time().less_than(&2));

    }).unwrap();
}
```
**Expected Output:**
```
--- Time 1: Inserting two names ---
Output: (("alice"), 1, 1)
Output: (("bob"), 1, 1)

--- Time 2: Inserting one name, removing another ---
Output: (("charlie"), 2, 1)
Output: (("alice"), 2, -1)
```

---

### Tutorial 2: The Core Operators - `map`, `filter`, `concat`

These are the simple, record-by-record transformations.

**Concepts:**
*   `.map(|data| ...)`: Transforms each record.
*   `.filter(|data| ...)`: Keeps or discards records based on a condition.
*   `.concat(&other_collection)`: Merges two collections into one.

**Code (`tutorial_2.rs`):**
```rust
use timely::dataflow::operators::{Input, Inspect};
use differential_dataflow::input::InputSession;
use differential_dataflow::operators::{Map, Filter, Concat};

fn main() {
    timely::execute::execute_from_args(std::env::args(), |worker| {
        let mut input1 = InputSession::new();
        let mut input2 = InputSession::new();

        worker.dataflow(|scope| {
            let collection1 = input1.to_collection(scope);
            let collection2 = input2.to_collection(scope);

            // A dataflow that takes names, makes them uppercase, and keeps only long ones.
            let stream1 = collection1
                .map(|name: String| name.to_uppercase())
                .filter(|name| name.len() > 4);

            // A different dataflow.
            let stream2 = collection2
                .map(|name: String| format!("Guest: {}", name));

            // Merge the results of both streams.
            stream1.concat(&stream2)
                   .inspect(|change| println!("Output: {:?}", change));
        });

        input1.insert("alice".to_string()); // Becomes "ALICE", kept
        input1.insert("bob".to_string());   // Becomes "BOB", filtered out
        input2.insert("david".to_string()); // Becomes "Guest: david"
        
        input1.advance_to(1);
        input2.advance_to(1);
        input1.flush();
        input2.flush();
        worker.step_while(|| input1.time().less_than(&1));

    }).unwrap();
}
```
**Expected Output:**
```
Output: (("ALICE"), 1, 1)
Output: (("Guest: david"), 1, 1)
```

#### **Exercise 1: Simple Transformation**
Given a stream of `(UserID, Age)` tuples, create a dataflow that outputs only the `UserID`s of users who are 18 or older.

---

### Tutorial 3: The First "Magic" - Aggregation with `count`

This is where ddflow starts to think differently. `count` operates on the stream of changes to maintain an up-to-date count.

**Concepts:**
*   `.count()`: Groups records and counts them.
*   **Retract-and-Emit:** When an input changes, `count` first *retracts* its old output (`-1` diff) and then *emits* its new output (`+1` diff).

**Code (`tutorial_3.rs`):**
```rust
use timely::dataflow::operators::{Input, Inspect};
use differential_dataflow::input::InputSession;
use differential_dataflow::operators::Count;

fn main() {
    timely::execute::execute_from_args(std::env::args(), |worker| {
        let mut input = InputSession::new();

        worker.dataflow(|scope| {
            input.to_collection(scope)
                 .map(|fruit: String| fruit) // Group by the fruit name itself
                 .count()
                 .inspect(|change| println!("Count Change: {:?}", change));
        });

        println!("--- Time 1: Adding 'apple' ---");
        input.insert("apple".to_string());
        input.advance_to(1);
        input.flush();
        worker.step_while(|| input.time().less_than(&1));

        println!("\n--- Time 2: Adding another 'apple' ---");
        input.insert("apple".to_string());
        input.advance_to(2);
        input.flush();
        worker.step_while(|| input.time().less_than(&2));

        println!("\n--- Time 3: Adding 'orange' ---");
        input.insert("orange".to_string());
        input.advance_to(3);
        input.flush();
        worker.step_while(|| input.time().less_than(&3));
    }).unwrap();
}
```
**Expected Output (The "Aha!" moment):**
```
--- Time 1: Adding 'apple' ---
Count Change: ((("apple"), 1), 1, 1)         // Apple count is now 1

--- Time 2: Adding another 'apple' ---
Count Change: ((("apple"), 1), 2, -1)        // Apple count is NO LONGER 1 (retraction)
Count Change: ((("apple"), 2), 2, 1)         // Apple count is NOW 2 (emission)

--- Time 3: Adding 'orange' ---
Count Change: ((("orange"), 1), 3, 1)        // Orange count is now 1
```

#### **Exercise 2: Simple Aggregation**
You have two input streams: `hires` and `fires`, both producing `DepartmentID` strings. Create a dataflow that maintains a live count of the number of employees in each department. `hires.concat(&fires)` is not the right model, because `fires` should decrement the count. Think about the `diff`s!

---

### Tutorial 4: The Heart of Relational Logic - `join`

This is the most critical operator for Datalog. We will focus on the performant pattern.

**Concepts:**
*   `.arrange_by_key()`: Creates a persistent, indexed "phone book" of a collection for fast lookups. **This is for stable, re-used data.**
*   `.join_core()`: The high-performance join operator that uses an arrangement.

**Code (`tutorial_4.rs`):**
```rust
use timely::dataflow::operators::{Input, Inspect};
use differential_dataflow::input::InputSession;
use differential_dataflow::operators::*;
use differential_dataflow::operators::arrange::ArrangeByKey;

// (EmployeeID, EmployeeName)
type Employee = (u32, String);
// (EmployeeID, DepartmentID)
type Assignment = (u32, u32);
// (DepartmentID, DepartmentName)
type Department = (u32, String);

fn main() {
    timely::execute::execute_from_args(std::env::args(), |worker| {
        let mut employees_in = InputSession::new();
        let mut assignments_in = InputSession::new();
        let mut departments_in = InputSession::new();

        worker.dataflow(|scope| {
            let employees = employees_in.to_collection(scope);
            let assignments = assignments_in.to_collection(scope);
            let departments = departments_in.to_collection(scope);

            // ARRANGE the "lookup tables" once for performance.
            // Key employees by EmployeeID.
            let employees_arranged = employees.map(|(id, name)| (id, name)).arrange_by_key();
            // Key departments by DepartmentID.
            let departments_arranged = departments.map(|(id, name)| (id, name)).arrange_by_key();

            // Join assignments with employees to get (DeptID, EmployeeName)
            let emp_with_dept_id = assignments
                .map(|(emp_id, dept_id)| (emp_id, dept_id)) // key by EmployeeID
                .join_core(&employees_arranged, |_emp_id, dept_id, emp_name| {
                    Some((dept_id.clone(), emp_name.clone()))
                });

            // Join that result with departments to get the final (DeptName, EmployeeName)
            emp_with_dept_id
                .join_core(&departments_arranged, |_dept_id, emp_name, dept_name| {
                    Some((dept_name.clone(), emp_name.clone()))
                })
                .inspect(|change| println!("Roster: {:?}", change));
        });

        employees_in.insert((1, "alice".to_string()));
        departments_in.insert((101, "Engineering".to_string()));
        assignments_in.insert((1, 101));

        employees_in.advance_to(1);
        assignments_in.advance_to(1);
        departments_in.advance_to(1);
        // ... flush and step ...
    }).unwrap();
}
```
**Expected Output:**
```
Roster: ((("Engineering", "alice")), 1, 1)
```

#### **Exercise 3: Relational Join**
You have three inputs: `Product(ProductID, Name)`, `Sale(SaleID, ProductID, Amount)`, and `Customer(CustomerID, SaleID)`. Create a dataflow that outputs `(CustomerName, ProductName, Amount)`. (This will require you to get customer names from another table!)

---

### Tutorial 5: The Final Boss - Recursion with `iterate`

This brings it all together to solve transitive closure.

**Concepts:**
*   `.iterate()`: The operator for creating safe, iterative feedback loops.
*   **Semi-Naive Evaluation Pattern:** The most robust way to write recursion.
    1.  Arrange the stable input (`Edge`) **outside** the loop.
    2.  Seed the iteration with the base cases (`Edge`).
    3.  Inside the loop, join the `frontier` (the *new* results from the last iteration) with the stable arrangement.

**Code (`tutorial_5.rs`):**
```rust
use timely::dataflow::operators::{Input, Inspect, Probe};
use differential_dataflow::input::InputSession;
use differential_dataflow::operators::*;
use differential_dataflow::operators::arrange::ArrangeByKey;

// This is the final, correct code we arrived at in our conversation.
fn main() {
    timely::execute::execute_from_args(std::env::args(), |worker| {
        let mut links_in = InputSession::new();
        let mut probe = ProbeHandle::new();
        worker.dataflow(|scope| {
            let links = links_in.to_collection(scope);

            // 1. Arrange the stable input ONCE, outside the loop.
            let arranged_links = links.arrange_by_key();

            // 2. Seed the iteration with the base cases (`links`).
            links.iterate(|frontier| {
                // 3. Bring the arrangement into the loop's scope.
                let arranged_links_inner = arranged_links.enter(&frontier.scope());

                // 4. Join the `frontier` (new paths) with the stable links.
                frontier
                    .map(|(x, y)| (y, x)) // key frontier by destination `y`
                    .join_core(&arranged_links_inner, |_y, &x, &z| Some((x, z)))
            })
            .distinct()
            .inspect(|change| println!("Path: {:?}", change))
            .probe_with(&mut probe);
        });
        
        links_in.insert((1, 2));
        links_in.insert((2, 3));
        links_in.advance_to(1);
        links_in.flush();
        worker.step_while(|| probe.less_than(links_in.time()));
    }).unwrap();
}
```
**Expected Output:**
```
Path: (((1, 2)), 1, 1)
Path: (((2, 3)), 1, 1)
Path: (((1, 3)), 1, 1)
```

#### **Exercise 4 (Challenge): Management Chain**
You have one input: `ReportsTo(EmployeeID, ManagerID)`. Create a dataflow that computes `AllSuperiors(EmployeeID, SuperiorID)`, finding all direct and indirect managers for every employee. This is just transitive closure with different names.

---

Take your time with these. Build and run each one. Then, try the exercises. This bottom-up approach will build the solid mental model you need to confidently write your interpreter's execution engine. Good luck