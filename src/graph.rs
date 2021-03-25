use {
    crate::logical_unit::{Id, LogicalUnit},
    petgraph::{graph::NodeIndex, Directed, Graph},
    std::collections::HashMap,
};

pub fn of_units(units: &Vec<LogicalUnit>) -> Graph<&LogicalUnit, (), Directed> {
    let mut graph: Graph<&LogicalUnit, (), Directed> = Graph::new();
    // map from unit id to the unit and its index in the graph (if it's been added)
    let mut map: HashMap<Id, (&LogicalUnit, Option<NodeIndex>)> = HashMap::new();
    for u in units {
        map.insert(u.id.clone(), (u, None));
    }
    for u in units {
        let idx = graph.add_node(u);
        map.insert(u.id.clone(), (&u, Some(idx)));
        if let Some(parent_id) = u.parent_id() {
            let parent_idx = {
                let &(parent, idx_opt) =
                    // Otherwise our graph database is corrupted
                    map.get(&parent_id).expect(&format!("parent of unit {} must be in map", u));
                match idx_opt {
                    Some(i) => i,
                    None => {
                        let i = graph.add_node(parent);
                        map.insert(parent.id.clone(), (parent, Some(i)));
                        i
                    }
                }
            };
            graph.add_edge(idx, parent_idx, ());
        }
    }
    graph
}

#[cfg(test)]
mod test {
    use {super::*, crate::logical_unit::Kind};

    #[test]
    fn can_construct_graph_of_units() {
        let units = vec![
            LogicalUnit::new(None, None, None, Kind::Requirement, "FOO.1", "").unwrap(),
            LogicalUnit::new(None, None, None, Kind::Requirement, "FOO.1::BAR.1", "").unwrap(),
            LogicalUnit::new(
                None,
                None,
                None,
                Kind::Requirement,
                "FOO.1::BAR.1::BAZ.1",
                "",
            )
            .unwrap(),
            LogicalUnit::new(None, None, None, Kind::Requirement, "FIZ.1", "").unwrap(),
        ];

        let graph = of_units(&units);
        assert_eq!(graph.node_count(), units.len());
        assert_eq!(graph.edge_count(), 2);
    }
}
