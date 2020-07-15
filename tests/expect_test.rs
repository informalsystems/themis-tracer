use assert_cmd::Command;
use glob::glob;
use std::fs;
use std::path::Path;

#[test]
fn mdx_tests() {
    let test_artifacts_dir = Path::new("target/test-artifacts");

    fs::create_dir_all(test_artifacts_dir).unwrap();

    for expect_file in glob("./tests/*.md").expect("Failed to read glob pattern") {
        let expected = expect_file.unwrap();
        let corrected = test_artifacts_dir.join(
            expected
                .strip_prefix("tests")
                .unwrap()
                .with_extension("md.corrected"),
        );

        Command::new("opam")
            .arg("exec")
            .arg("--")
            .arg("ocaml-mdx")
            .arg("test")
            .arg(expected.to_owned())
            .arg("--output")
            .arg(corrected.to_owned())
            .assert()
            .success();

        Command::new("diff")
            .arg("--color")
            .arg(expected)
            .arg(corrected)
            .assert()
            .success();
    }
}
