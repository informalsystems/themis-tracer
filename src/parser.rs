#[derive(Debug, PartialEq, Eq)]
pub enum UnitRefSearch {
    Text(String),
    Ref(String),
}

// Since peg doesn't allow for composing rule sets, we put all the parsers in
// one place so we can reuse common rules
peg::parser! {
    pub grammar parser() for str {
        pub rule letter() -> String =
            l:$(['a'..='z' | 'A'..='Z' ])
        { l.to_string() }

        pub rule init_char() -> String =
            c:$(letter() / ['_'])
        { c.to_string() }

        pub rule digit() -> String =
            d:$(['0'..='9'])
        { d.to_string() }

        pub rule logical_unit_definiendum() -> String =
            "|" tag:$(logical_unit_id()) "|"
        { tag.to_string() }

        pub rule logical_unit_ref() -> String =
            "[" tag:$(logical_unit_id()) "]"
        { tag.to_string() }

        pub rule logical_unit_id() -> Vec<(String, u32)> =
            id:(luid_part() ** "::")
        { id }

        pub rule find_logical_unit_refs() -> Option<Vec<UnitRefSearch>> =
            res:ref_search_result()*
        { if res.iter().any(|i| matches!(i, UnitRefSearch::Ref(_))) {
            Some(res)
          } else {
            None
          }
        }

        rule ref_search_result() -> UnitRefSearch =
            f:(non_ref_text_found() / ref_found())
        { f }

        rule non_ref_text_found() -> UnitRefSearch =
             t:$((!logical_unit_ref() [_])+)
        { UnitRefSearch::Text(t.to_string()) }

        rule ref_found() -> UnitRefSearch =
            r:logical_unit_ref()
        { UnitRefSearch::Ref(r) }

        rule luid_tag() -> String =
            t:$(init_char() (init_char() / digit() / "-")+)
        { t.to_string() }

        rule luid_version() -> u32 =
            v:$(['1'..='9'] digit()*)
        { v.parse().unwrap() }

        rule luid_part() -> (String, u32) =
            t:luid_tag() "." v:luid_version()
        { (t, v) }
    }
}

#[cfg(test)]
mod test_parser {
    use super::*;
    #[test]
    fn can_parse_id() {
        assert_eq!(
            parser::logical_unit_id("FOO.1::BAR-BAZ.2::BING.3").unwrap(),
            vec![
                ("FOO".to_string(), 1),
                ("BAR-BAZ".to_string(), 2),
                ("BING".to_string(), 3)
            ]
        )
    }

    #[test]
    fn can_parse_defininiendum() {
        assert_eq!(
            "FOO.1::BAR.1::BAZ-BOP.1".to_string(),
            parser::logical_unit_definiendum(&"|FOO.1::BAR.1::BAZ-BOP.1|").unwrap()
        )
    }

    #[test]
    fn find_logical_unit_refs_can_extract_unit_refs_from_text() {
        let expected: Vec<UnitRefSearch> = vec![
            UnitRefSearch::Text("Some text here ".into()),
            UnitRefSearch::Ref("FOO.1::BAR.1".into()),
            UnitRefSearch::Text(" more text ".into()),
            UnitRefSearch::Ref("FIZ.1".into()),
        ];
        let actual =
            parser::find_logical_unit_refs(&"Some text here [FOO.1::BAR.1] more text [FIZ.1]")
                .unwrap();
        assert_eq!(expected, actual.unwrap())
    }

    #[test]
    fn find_logical_unit_refs_with_no_refs_is_none() {
        // Ensures we don't error on the empty string
        let actual = parser::find_logical_unit_refs(&"Some text here but now ref").unwrap();
        assert_eq!(None, actual)
    }

    #[test]
    fn find_logical_unit_refs_does_not_error_on_empty_string() {
        let actual = parser::find_logical_unit_refs(&"").unwrap();
        assert_eq!(None, actual)
    }
}
