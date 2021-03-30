use {
    crate::{graph::UnitGraph, logical_unit::LogicalUnit},
    anyhow::Result,
    petgraph::{graph::NodeIndex, Direction},
    std::{fmt, io::Write},
};

#[derive(Clone, Debug)]
enum Html {
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
    ($tag:literal, $attrs:expr, $($child:expr);*) => {
        { let mut children = Vec::new();
          $(
              children.push($child);
          )*
          Html::Tag($tag.to_string(), $attrs, children) }
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

fn dl_tree<W: Write>(parent_idx: NodeIndex<u32>, graph: &UnitGraph, writer: &mut W) -> Result<()> {
    let parent = graph.node_weight(parent_idx).unwrap();
    let mut children = graph
        .neighbors_directed(parent_idx, Direction::Outgoing)
        .peekable();
    write!(
        writer,
        r#"<dt id="{0}"><strong>{0}</strong></dt>\n"#,
        parent.id
    )?;
    writeln!(writer, r#"<dd><p class="content">{}</p>"#, parent.content)?;
    if let Some(_) = children.peek() {
        write!(
            writer,
            r#"<details class="implementations"><summary>Implemented by...</summary>\n<dl>"#
        )?;
        for child in children {
            dl_tree(child, graph, writer)?;
        }
        writeln!(writer, "</dl>\n</details>")?;
    }
    writeln!(writer, "</dd>")?;
    Ok(())
}

pub fn unit_tree<W: Write>(graph: &UnitGraph, writer: &mut W) -> Result<()> {
    let source_idxs = graph.externals(Direction::Incoming);
    writer.write(
        r#"
<html>
<head>
<title>Context</title>
</head>
<body>
<dl>
"#
        .as_bytes(),
    )?;
    for i in source_idxs {
        dl_tree(i, graph, writer)?
    }
    writer.write("</dl>\n</body>\n</html>\n".as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod test {
    use {super::*, crate::graph, kuchiki, kuchiki::traits::TendrilSink};

    #[test]
    fn can_generate_dl_tree() {
        let expected = r##"<html><head>
<title>Context</title>
</head>
<body>
<dl>
<dt id="FOO.1"><strong>FOO.1</strong></dt>\n<dd><p class="content">Foo content</p>
<details class="implementations"><summary>Implemented by...</summary>\n<dl><dt id="FOO.1::BING.1"><strong>FOO.1::BING.1</strong></dt>\n<dd><p class="content">Bing content</p>
</dd>
<dt id="FOO.1::BAR.1"><strong>FOO.1::BAR.1</strong></dt>\n<dd><p class="content">Bar content</p>
<details class="implementations"><summary>Implemented by...</summary>\n<dl><dt id="FOO.1::BAR.1::BAZ.1"><strong>FOO.1::BAR.1::BAZ.1</strong></dt>\n<dd><p class="content">Baz content</p>
</dd>
</dl>
</details>
</dd>
</dl>
</details>
</dd>
<dt id="FIZ.1"><strong>FIZ.1</strong></dt>\n<dd><p class="content">Fiz content</p>
</dd>
</dl>


</body></html>"##.to_string();
        let mut writer = Vec::new();
        let units = graph::test::test_units();
        let graph = graph::of_units(&units);
        unit_tree(&graph, &mut writer).unwrap();

        let mut formatted = Vec::new();
        kuchiki::parse_html()
            .from_utf8()
            .read_from(&mut writer.as_slice())
            .unwrap()
            .serialize(&mut formatted)
            .unwrap();
        let actual = String::from_utf8_lossy(&formatted);

        println!("Expected:\n{}", expected);
        println!("Actual:\n{}", actual);
        assert_eq!(expected, actual)
    }

    #[test]
    fn can_write_html() {
        let html = tag!(
            "html",
            vec![],
            tag!("head", vec![], tag!("title", vec![], txt!("foo")));
            tag!(
                "body",
                vec![attr!("class", "class1 class2")],
                tag!("p", vec![], txt!("Some content"))
            )
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
