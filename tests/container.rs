use std::env;
use std::fs;
use std::path::PathBuf;

use hoshidicts::{Error, index, pack, verify};

fn scratch(name: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("hoshidicts-rs-{name}"));
    fs::remove_dir_all(&dir).ok();
    fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

fn message(error: Error) -> String {
    match error {
        Error::Container(message) => message,
        other => panic!("expected Error::Container, got {other:?}"),
    }
}

#[test]
fn pack_reports_why_a_directory_is_not_a_dictionary_and_leaves_nothing_behind() {
    let dir = scratch("pack-not-a-dictionary");
    let output = dir.join("out.hoshi");

    let error = message(pack(&dir, &output).expect_err("pack should fail"));
    assert!(error.contains("payload version"), "{error}");
    assert!(!output.exists());
    assert!(!dir.join("out.hoshi.tmp").exists());
}

#[test]
fn verify_and_index_reject_a_file_that_is_not_a_container() {
    let dir = scratch("not-a-container");

    let short = dir.join("short.hoshi");
    fs::write(&short, b"too short").expect("write");
    let truncated = message(verify(&short).expect_err("verify should fail"));
    assert!(
        truncated.contains("truncated container header"),
        "{truncated}"
    );

    let path = dir.join("wrong-magic.hoshi");
    fs::write(&path, vec![b'x'; 128]).expect("write");
    let from_verify = message(verify(&path).expect_err("verify should fail"));
    let from_index = message(index(&path).expect_err("index should fail"));
    assert!(
        from_verify.contains("not a .hoshi container"),
        "{from_verify}"
    );
    assert!(
        from_index.contains("not a .hoshi container"),
        "{from_index}"
    );
}
