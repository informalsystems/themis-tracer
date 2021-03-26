use {
    crate::logical_unit::{Id, LogicalUnit},
    petgraph::{
        dot::{Config, Dot},
        graph::NodeIndex,
        Directed, Graph,
    },
    std::collections::HashMap,
};

type UnitGraph<'a> = Graph<&'a LogicalUnit, (), Directed>;

pub fn of_units(units: &Vec<LogicalUnit>) -> UnitGraph {
    let mut graph: UnitGraph = Graph::new();
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
            graph.add_edge(parent_idx, idx, ());
        }
    }
    graph
}

pub fn unit_as_dot(base_url: &str, graph: &UnitGraph) -> String {
    format!(
        "{:?}",
        Dot::with_attr_getters(
            graph,
            &[Config::NodeNoLabel, Config::EdgeNoLabel],
            &|_graph, _| "".to_string(),
            &|_graph, (_idx, unit)| {
                format!(
                    r##"label="{id}" tooltip="{content}" href="{url}#{id}" "##,
                    id = unit.id,
                    content = unit.content,
                    url = base_url
                )
            },
        )
    )
}

#[cfg(test)]
mod test {
    use {super::*, crate::logical_unit::Kind};

    #[test]
    fn can_construct_graph_of_units() {
        let units = vec![
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
            LogicalUnit::new(None, None, None, Kind::Requirement, "FIZ.1", "Fiz content").unwrap(),
        ];

        let expected = r#"digraph {
    0 [ label="FOO.1" tooltip="Coo content" href="just/a/test#FOO.1" ]
    1 [ label="FOO.1::BAR.1" tooltip="Bar content" href="just/a/test#FOO.1::BAR.1" ]
    2 [ label="FOO.1::BAR.1::BAZ.1" tooltip="Baz content" href="just/a/test#FOO.1::BAR.1::BAZ.1" ]
    3 [ label="FIZ.1" tooltip="Fiz content" href="just/a/test#FIZ.1" ]
    0 -> 1 [ ]
    1 -> 2 [ ]
}
"#;
        let graph = of_units(&units);
        let dot_graph = unit_as_dot("just/a/test", &graph);
        assert_eq!(expected, dot_graph);
    }
}
