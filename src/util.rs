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
            "|" d:$((init_char() / digit() / "." / "::")+) "|"
        { d.to_string() }

        pub rule logical_unit_id() -> Vec<(String, u32)> =
            id:(luid_part() ** "::")
        { id }

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
            parser::logical_unit_id("FOO.1::BAR-BAZ.2::BING.3"),
            Ok(vec![
                ("FOO".to_string(), 1),
                ("BAR-BAZ".to_string(), 2),
                ("BING".to_string(), 3)
            ])
        )
    }

    #[test]
    fn can_parse_defininiendum() {
        assert_eq!(
            Ok("FOO.1::BAR.1::BAZ.1".to_string()),
            parser::logical_unit_definiendum(&"|FOO.1::BAR.1::BAZ.1|")
        )
    }
}
