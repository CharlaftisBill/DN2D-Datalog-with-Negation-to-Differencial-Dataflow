use differential_dataflow::input::InputSession;
use differential_dataflow::operators::*;
use differential_dataflow::operators::arrange::ArrangeByKey;
use serde::{Serialize, Deserialize}; // <-- ADD THIS LINE

// --- Type Aliases ---
type User = String;
type Resource = String;
type AccessLevel = String;
type Group = String;

// --- The Unifying Enum with Serialization ---
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // <-- ADD SERIALIZE/DESERIALIZE
enum AccessFact {
    Perm((User, Resource, AccessLevel)),
    Access((User, Resource)),
}

fn main() {
    timely::execute::execute_from_args(std::env::args(), |worker| {
        // ... (input sessions are the same)
        let mut direct_perms_in = InputSession::<i32, (User, Resource, AccessLevel), isize>::new();
        let mut group_members_in = InputSession::<i32, (User, Group), isize>::new();
        let mut group_perms_in = InputSession::<i32, (Group, Resource, AccessLevel), isize>::new();
        let mut hierarchy_in = InputSession::<i32, (Resource, Resource), isize>::new();
        let mut tag_policies_in = InputSession::<i32, (String, String, String, AccessLevel), isize>::new();
        let mut resource_tags_in = InputSession::<i32, (Resource, String, String), isize>::new();
        let mut is_public_in = InputSession::<i32, Resource, isize>::new();
        let mut probe = timely::dataflow::ProbeHandle::new();

        worker.dataflow(|scope| {
            // ... (collections are the same)
            let direct_perms = direct_perms_in.to_collection(scope);
            let group_members = group_members_in.to_collection(scope);
            let group_perms = group_perms_in.to_collection(scope);
            let resource_hierarchy = hierarchy_in.to_collection(scope);
            let tag_policies = tag_policies_in.to_collection(scope);
            let resource_tags = resource_tags_in.to_collection(scope);
            let is_public = is_public_in.to_collection(scope);

            // --- Section 2: First Recursive Block (Correct as is) ---
            let contained_in = resource_hierarchy.iterate(|closure| {
                let resource_hierarchy = resource_hierarchy.enter(&closure.scope());
                let arranged_hierarchy = resource_hierarchy.arrange_by_key();
                closure.map(|(c, p)| (p, c)).join_core(&arranged_hierarchy, |_p, c, gp| Some((c.clone(), gp.clone()))).concat(&resource_hierarchy).distinct()
            });

            // Arrange `contained_in` ONCE for high-performance joins later.
            let arranged_contained_in_by_child = contained_in.map(|(c, p)| (c, p)).arrange_by_key();
            let arranged_contained_in_by_parent = contained_in.map(|(c, p)| (p, c)).arrange_by_key();

            // --- Section 3: Second, Larger Recursive Block ---
            let from_groups = group_members.map(|(u, g)| (g, u)).join(&group_perms.map(|(g,r,l)| (g, (r,l)))).map(|(_g, (u, (r, l)))| (u, r, l));
            let from_tags = tag_policies.map(|(_p, k, v, l)| ((k, v), l)).join(&resource_tags.map(|(r, k, v)| ((k, v), r))).map(|((_k, _v), (l, r))| ("any_user_placeholder".to_string(), r, l));
            let user_permission_base = direct_perms.concat(&from_groups).concat(&from_tags).distinct();
            let public_access_base = is_public.map(|r| ("any_user_placeholder".to_string(), r));
            
            let initial_facts = user_permission_base.map(AccessFact::Perm)
                .concat(&public_access_base.map(AccessFact::Access));

            let all_facts = initial_facts.iterate(|frontier| {
                let arranged_contained_in_by_child = arranged_contained_in_by_child.enter(&frontier.scope());
                let arranged_contained_in_by_parent = arranged_contained_in_by_parent.enter(&frontier.scope());
                
                let perm_delta = frontier.filter(|fact| matches!(fact, AccessFact::Perm(_)))
                                         .map(|fact| if let AccessFact::Perm(p) = fact { p } else { unreachable!() });
                let access_delta = frontier.filter(|fact| matches!(fact, AccessFact::Access(_)))
                                           .map(|fact| if let AccessFact::Access(a) = fact { a } else { unreachable!() });

                let inherited_perms = perm_delta.map(|(u, p, l)| (p, (u, l))).join_core(&arranged_contained_in_by_parent, |_p, ul, c| Some((ul.0.clone(), c.clone(), ul.1.clone())));
                let admin_from_child = access_delta.map(|(u, c)| (c, u)).join_core(&arranged_contained_in_by_child, |_c, u, p| Some((u.clone(), p.clone(), "Admin".to_string())));
                let access_from_perms = perm_delta.map(|(u, r, _l)| (u, r));

                inherited_perms.map(AccessFact::Perm)
                    .concat(&admin_from_child.map(AccessFact::Perm))
                    .concat(&access_from_perms.map(AccessFact::Access))
                    .distinct()
            });

            let effective_permission_result = all_facts.filter(|fact| matches!(fact, AccessFact::Perm(_)))
                                                       .map(|fact| if let AccessFact::Perm(p) = fact { p } else { unreachable!() });
            let has_access_result = all_facts.filter(|fact| matches!(fact, AccessFact::Access(_)))
                                             .map(|fact| if let AccessFact::Access(a) = fact { a } else { unreachable!() });

            // --- Section 4 & 5 (Correct as is) ---
            let admin_counts = effective_permission_result.filter(|(_u, _r, l)| l == "Admin").map(|(u, _r, _l)| u).count();
            let privileged_users = group_members.map(|(u, _g)| u).count().filter(|(_u, c)| *c > 5).map(|(u, _c)| u);
            let prod_db_access = has_access_result.filter(|(_u, r)| r == "prod-billing-db").map(|(u, _r)| u);
                
            admin_counts.inspect(|x| println!("Admin Count: {:?}", x));
            privileged_users.inspect(|x| println!("Privileged User: {:?}", x));
            prod_db_access.inspect(|x| println!("Prod DB Access: {:?}", x));
            prod_db_access.probe_with(&mut probe);
        });
        
        // --- Driver (Correct as is) ---
        println!("--- Loading initial state from CSVs ---");
        // ... (data loading is the same)
        direct_perms_in.insert(("alice".to_string(), "dev-vm".to_string(), "Admin".to_string()));
        direct_perms_in.insert(("charlie".to_string(), "dev-vm".to_string(), "ReadOnly".to_string()));
        group_members_in.insert(("alice".to_string(), "developers".to_string()));
        group_members_in.insert(("bob".to_string(), "admins".to_string()));
        group_members_in.insert(("charlie".to_string(), "developers".to_string()));
        group_perms_in.insert(("developers".to_string(), "project-alpha".to_string(), "ReadOnly".to_string()));
        group_perms_in.insert(("admins".to_string(), "org-main".to_string(), "Admin".to_string()));
        hierarchy_in.insert(("prod-billing-db".to_string(), "project-alpha".to_string()));
        hierarchy_in.insert(("dev-vm".to_string(), "project-alpha".to_string()));
        hierarchy_in.insert(("project-alpha".to_string(), "org-main".to_string()));
        tag_policies_in.insert(("billing_admin_policy".to_string(), "department".to_string(), "billing".to_string(), "Admin".to_string()));
        resource_tags_in.insert(("prod-billing-db".to_string(), "department".to_string(), "billing".to_string()));
        resource_tags_in.insert(("dev-vm".to_string(), "environment".to_string(), "dev".to_string()));
        is_public_in.insert("public-landing-page".to_string());
        direct_perms_in.advance_to(1);
        group_members_in.advance_to(1);
        group_perms_in.advance_to(1);
        hierarchy_in.advance_to(1);
        tag_policies_in.advance_to(1);
        resource_tags_in.advance_to(1);
        is_public_in.advance_to(1);
        direct_perms_in.flush();
        group_members_in.flush();
        group_perms_in.flush();
        hierarchy_in.flush();
        tag_policies_in.flush();
        resource_tags_in.flush();
        is_public_in.flush();
        while probe.less_than(direct_perms_in.time()) { worker.step(); }
        println!("\n--- Computation Complete ---");
    }).unwrap();
}