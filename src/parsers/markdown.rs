//!
//! Markdown parsing interface for Themis Tracer.
//!

#[cfg(test)]
mod test {
    use super::*;
    use textwrap::dedent;

    const SIMPLE_SPEC: &str = r#"
    # Specification

    |SPEC-HELLO.1|
    :   When executed, the program must print out the text "Hello world!"
    "#;

    const MULTI_UNIT_SPEC: &str = r#"
    # Specification

    |SPEC-INPUT.1|
    :   When executed, the program must print the text: "Hello! What's your name?",
        and allow the user to input their name.
    
    |SPEC-HELLO.2|
    :   Once the user's name has been obtained, the program must print out the text
        "Hello {name}!", where `{name}` must be replaced by the name obtained in
        [SPEC-INPUT.1].
    "#;

    fn test_simple_spec_parsing() {}
}
