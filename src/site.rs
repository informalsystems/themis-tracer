use {
    crate::graph::UnitGraph,
    petgraph::{graph::NodeIndex, Direction},
    std::fmt,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Html {
    Tag(String, Vec<(String, String)>, Vec<Html>),
    Text(String),
}

fn indent_n(f: &mut fmt::Formatter<'_>, n: u32) -> fmt::Result {
    for _ in 0..n {
        write!(f, " ")?;
    }
    Ok(())
}

fn attrs_to_string(attrs: &[(String, String)]) -> String {
    attrs
        .iter()
        .map(|(a, v)| format!(r#"{}="{}""#, a, v))
        .collect::<Vec<String>>()
        .join(" ")
}

macro_rules! tag {
    ($tag:literal, $attrs:expr, $children:expr) => {
        Html::Tag($tag.to_string(), $attrs, $children)
    };
}

macro_rules! txt {
    ($txt:expr) => {
        Html::Text($txt.to_string())
    };
}

macro_rules! attr {
    ($attr:literal, $value:expr) => {
        ($attr.to_string(), $value.to_string())
    };
}

impl Html {
    fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, indent: u32) -> fmt::Result {
        match self {
            Html::Tag(tag, attrs, inner) => {
                indent_n(f, indent)?;
                writeln!(
                    f,
                    "<{tag} {attrs}>",
                    tag = tag,
                    attrs = attrs_to_string(attrs)
                )?;
                for i in inner.iter() {
                    i.fmt_indented(f, indent + 2)?;
                }
                indent_n(f, indent)?;
                writeln!(f, "</{}>", tag)
            }
            Html::Text(text) => {
                indent_n(f, indent + 2)?;
                writeln!(f, "{}", text)
            }
        }
    }
}

impl<'a> fmt::Display for Html {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, 0)
    }
}

impl From<&UnitGraph<'_>> for Html {
    fn from(graph: &UnitGraph) -> Html {
        let unit_tree: Vec<Html> = graph
            .externals(Direction::Incoming)
            .flat_map(|i| unit_tree_html(i, graph))
            .collect();

        tag!(
            "html",
            vec![],
            vec![
                tag!(
                    "head",
                    vec![],
                    vec![tag!("title", vec![], vec![txt!("Context")])]
                ),
                tag!("body", vec![], vec![tag!("dl", vec![], unit_tree)])
            ]
        )
    }
}

fn unit_tree_html(parent_idx: NodeIndex<u32>, graph: &UnitGraph) -> Vec<Html> {
    let parent = graph.node_weight(parent_idx).unwrap();

    let content = tag!(
        "p",
        vec![attr!("class", "content")],
        vec![txt!(parent.content)]
    );

    let mut implementors: Vec<Html> = {
        let items: Vec<Html> = graph
            .neighbors_directed(parent_idx, Direction::Outgoing)
            .flat_map(|child| unit_tree_html(child, graph))
            .collect();
        if items.is_empty() {
            vec![]
        } else {
            vec![tag!(
                "details",
                vec![attr!("class", "implementations")],
                vec![
                    tag!("summary", vec![], vec![txt!("Implemented by...")]),
                    tag!("dl", vec![], items)
                ]
            )]
        }
    };

    let mut children = vec![content];
    children.append(&mut implementors);
    vec![
        tag!(
            "dt",
            vec![attr!("id", parent.id)],
            vec![tag!("strong", vec![], vec![txt!(parent.id)])]
        ),
        tag!("dd", vec![], children),
    ]
}

#[cfg(test)]
mod test {
    use {super::*, crate::graph};

    #[test]
    fn html_from_unit_graph() {
        let actual = Html::from(&graph::of_units(&graph::test::test_units()));
        let expected = r#"<html >
  <head >
    <title >
        Context
    </title>
  </head>
  <body >
    <dl >
      <dt id="FOO.1">
        <strong >
            FOO.1
        </strong>
      </dt>
      <dd >
        <p class="content">
            Foo content
        </p>
        <details class="implementations">
          <summary >
              Implemented by...
          </summary>
          <dl >
            <dt id="FOO.1::BING.1">
              <strong >
                  FOO.1::BING.1
              </strong>
            </dt>
            <dd >
              <p class="content">
                  Bing content
              </p>
            </dd>
            <dt id="FOO.1::BAR.1">
              <strong >
                  FOO.1::BAR.1
              </strong>
            </dt>
            <dd >
              <p class="content">
                  Bar content
              </p>
              <details class="implementations">
                <summary >
                    Implemented by...
                </summary>
                <dl >
                  <dt id="FOO.1::BAR.1::BAZ.1">
                    <strong >
                        FOO.1::BAR.1::BAZ.1
                    </strong>
                  </dt>
                  <dd >
                    <p class="content">
                        Baz content
                    </p>
                  </dd>
                </dl>
              </details>
            </dd>
          </dl>
        </details>
      </dd>
      <dt id="FIZ.1">
        <strong >
            FIZ.1
        </strong>
      </dt>
      <dd >
        <p class="content">
            Fiz content
        </p>
      </dd>
    </dl>
  </body>
</html>
"#
        .to_string();

        println!("Expected:\n{}", expected);
        println!("Actual:\n{}", actual);
        assert_eq!(expected, actual.to_string())
    }

    #[test]
    fn can_write_html() {
        let html = tag!(
            "html",
            vec![],
            vec![
                tag!(
                    "head",
                    vec![],
                    vec![tag!("title", vec![], vec![txt!("foo")])]
                ),
                tag!(
                    "body",
                    vec![attr!("class", "class1 class2")],
                    vec![tag!("p", vec![], vec![txt!("Some content")])]
                )
            ]
        );
        let actual = html.to_string();
        let expected = r#"<html >
  <head >
    <title >
        foo
    </title>
  </head>
  <body class="class1 class2">
    <p >
        Some content
    </p>
  </body>
</html>
"#
        .to_string();
        println!("Expected:\n{}", expected);
        println!("Actual:\n{}", actual);
        assert_eq!(expected, actual);
    }
}
