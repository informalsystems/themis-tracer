use assert_cmd::Command;
use glob::glob;
use std::path::Path;

#[test]
fn mdx_tests() {
    let test_artifacts_dir = Path::new("target/test-artifacts");

    let mut mdx = Command::new("ocaml-mdx");
    let mut diff = Command::new("diff");

    for expect_file in glob("./tests/*.md").expect("Failed to read glob pattern") {
        let expected = expect_file.unwrap();
        let corrected = test_artifacts_dir.join(
            expected
                .strip_prefix("tests")
                .unwrap()
                .with_extension("corrected"),
        );

        mdx.arg("test")
            .arg(expected.to_owned())
            .arg("--output")
            .arg(corrected.to_owned())
            .assert()
            .success();

        diff.arg("--color")
            .arg(expected)
            .arg(corrected)
            .assert()
            .success();
    }
}
