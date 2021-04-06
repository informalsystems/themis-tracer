use {
    crate::logical_unit::LogicalUnit,
    log,
    petgraph::{
        dot::{Config, Dot},
        stable_graph::{NodeIndex, StableGraph},
        Directed,
    },
    std::collections::BTreeMap,
};

pub type UnitGraph<'a> = StableGraph<&'a LogicalUnit, (), Directed>;
type UnitMap<'a> = BTreeMap<String, (&'a LogicalUnit, Option<NodeIndex<u32>>)>;

// (true, idx) if the node was inserted or (false, idx) if it was already present
fn try_insert_node<'a>(
    graph: &mut UnitGraph<'a>,
    map: &mut UnitMap<'a>,
    unit: &'a LogicalUnit,
) -> (bool, NodeIndex) {
    let tag = unit.id.to_string();
    match map.get(&tag).unwrap() {
        (_, Some(idx)) => (false, *idx),
        (_, None) => {
            let idx = graph.add_node(unit);
            map.insert(tag, (unit, Some(idx)));
            (true, idx)
        }
    }
}

pub fn of_units(units: &[LogicalUnit]) -> UnitGraph {
    log::debug!("generating graph of units");
    let mut graph: UnitGraph = StableGraph::new();
    // map from unit id to the unit and its index in the graph (if it's been added)
    let mut map = BTreeMap::new();
    for u in units {
        map.insert(u.id.to_string(), (u, None));
    }
    for u in units {
        // If we're adding the unit fresh to the graph
        if let (true, idx) = try_insert_node(&mut graph, &mut map, u) {
            if let Some(parent_id) = u.parent_id() {
                match map.get(&parent_id.to_string()) {
                    None => {
                        log::warn!(
                            "orphan unit {orphan} is missing its parent {parent}",
                            orphan = u.id,
                            parent = parent_id
                        );
                    }
                    Some(&(parent, _)) => {
                        let (_, parent_idx) = try_insert_node(&mut graph, &mut map, parent);
                        graph.add_edge(parent_idx, idx, ());
                    }
                };
            }
        }
        // We can unwrap here, because we know every unit is in the map
        // If the unit is not yet in the graph...
        // if let (_, None) = map.get(&u.id.to_string()).unwrap() {
        //     let idx = graph.add_node(u);
        //     // Record that it's added in the graph
        //     map.insert(u.id.to_string(), (&u, Some(idx)));
        //     // If the unit has a parentk...
        //     if let Some(parent_id) = u.parent_id() {
        //         // ... get the parent's index in the graph
        //         let parent_idx = {
        //             match map.get(&parent_id.to_string()) {
        //                 None => {
        //                     log::warn!(
        //                         "orphan unit: found {orphan} but it's parent {parent} is missing",
        //                         orphan = u.id,
        //                         parent = parent_id
        //                     );
        //                     None
        //                 }
        //                 Some(&(parent, idx_opt)) => match idx_opt {
        //                     // If the parent is already enterd, we retreive it's index in the graph
        //                     Some(i) => i,
        //                     // Otherwise, we enter it into the graph
        //                     None => {
        //                         let i = graph.add_node(parent);
        //                         map.insert(parent.id.to_string(), (parent, Some(i)));
        //                         i
        //                     }
        //                 },
        //             }
        //         };
        //         // Finally, add an edge from the parent to the child
        //         graph.add_edge(parent_idx, idx, ());
        //     }
        // }
    }
    graph
}

pub fn as_dot(base_url: &str, graph: &UnitGraph) -> String {
    log::debug!("gendering unit graph to dot");
    format!(
        "{:?}",
        Dot::with_attr_getters(
            graph,
            &[Config::NodeNoLabel, Config::EdgeNoLabel],
            &|_graph, _| "".to_string(),
            &|_graph, (_idx, unit)| {
                let content = unit.content.replace("\n", " ");
                // We render the content via debut for the string quoting
                format!(
                    r##"label="{id}" tooltip={:?} href="{url}#{id}" "##,
                    content,
                    id = unit.id,
                    url = base_url
                )
            },
        )
    )
}

#[cfg(test)]
pub(crate) mod test {
    use {super::*, crate::logical_unit::Kind};

    pub fn test_units() -> Vec<LogicalUnit> {
        vec![
            LogicalUnit::new(None, None, None, Kind::Requirement, "FOO.1", "Foo content").unwrap(),
            LogicalUnit::new(
                None,
                None,
                None,
                Kind::Requirement,
                "FOO.1::BAR.1",
                "Bar content",
            )
            .unwrap(),
            LogicalUnit::new(
                None,
                None,
                None,
                Kind::Requirement,
                "FOO.1::BAR.1::BAZ.1",
                "Baz content",
            )
            .unwrap(),
            LogicalUnit::new(
                None,
                None,
                None,
                Kind::Requirement,
                "FOO.1::BING.1",
                "Bing content",
            )
            .unwrap(),
            LogicalUnit::new(None, None, None, Kind::Requirement, "FIZ.1", "Fiz content").unwrap(),
        ]
    }

    #[test]
    fn can_construct_graph_of_units() {
        let expected = r#"digraph {
    0 [ label="FOO.1" tooltip="Foo content" href="just/a/test#FOO.1" ]
    1 [ label="FOO.1::BAR.1" tooltip="Bar content" href="just/a/test#FOO.1::BAR.1" ]
    2 [ label="FOO.1::BAR.1::BAZ.1" tooltip="Baz content" href="just/a/test#FOO.1::BAR.1::BAZ.1" ]
    3 [ label="FOO.1::BING.1" tooltip="Bing content" href="just/a/test#FOO.1::BING.1" ]
    4 [ label="FIZ.1" tooltip="Fiz content" href="just/a/test#FIZ.1" ]
    0 -> 1 [ ]
    1 -> 2 [ ]
    0 -> 3 [ ]
}
"#;
        let units = test_units();
        let graph = of_units(&units);
        let actual = as_dot("just/a/test", &graph);
        println!("Expected:\n{}", expected);
        println!("Actual: \n{}", actual);
        assert_eq!(expected, actual);
    }
}
