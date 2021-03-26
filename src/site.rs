use {
    crate::graph::UnitGraph,
    anyhow::Result,
    petgraph::{graph::NodeIndex, Direction},
    std::io::Write,
};

// TODO Use html construction lib?

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
}
