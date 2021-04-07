use assert_cmd::Command;
use glob::glob;
use std::fs;
use std::io;
use std::path::Path;
use std::{env, io::Write};

#[test]
fn mdx_tests() {
    env::set_current_dir(&Path::new("./tests")).unwrap();

    let test_sandbox_dir = Path::new("../target/test-sandbox");
    let test_repos_dir = Path::new("./repos");

    let test_artifacts_dir = Path::new("../target/test-artifacts");
    fs::create_dir_all(test_artifacts_dir).unwrap();

    // Add the build executable to the path
    let mut path = Path::new("../target/debug")
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    path.push(':');
    path.push_str(&env::var("PATH").unwrap());

    for expect_file in glob("*.md").expect("Failed to read glob pattern") {
        let expected = expect_file.unwrap();
        let corrected = test_artifacts_dir.join(expected.with_extension("md.corrected"));

        Command::new("opam")
            // Update the PATH var
            .env("PATH", &path)
            .env("TRACER_HOME", "../target/test-sandbox")
            .env("RUST_LOG", "error")
            .arg("exec")
            .arg("--")
            .arg("ocaml-mdx")
            .arg("test")
            .arg(expected.to_owned())
            // .arg("-s")
            // .arg("`add` repos to the current working context")
            .arg("--output")
            .arg(corrected.to_owned())
            .assert()
            .success();

        let output = Command::new("diff")
            .arg("--color")
            .arg(expected.clone())
            .arg(corrected)
            .output()
            .unwrap();

        if !output.status.success() {
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
            eprintln!("failure in {:?}", expected);
            assert!(false)
        }

        // Clean up sandbox dirs
        let _ = fs::remove_dir_all(test_repos_dir);
        let _ = fs::remove_dir_all(test_sandbox_dir);
    }
}
