mod utils;

use std::process::Command;

use utils::*;

use assert_cmd::prelude::*;
use git2::Repository;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_list() {
    let basedir = tempdir().unwrap();
    let repo = Repository::init(basedir.path()).unwrap();
    make_initial_commit(&repo);

    let first_commit =
        commit_random_file(basedir.path(), &repo, "ipsum-i", "first commit").unwrap();

    // create 2nd branch
    checkout_new_branch(&repo, &first_commit, "second-branch");
    commit_random_file(basedir.path(), &repo, "ipsum-ii", "second commit").unwrap();
    let second_commit =
        commit_random_file(basedir.path(), &repo, "ipsum-ii-a", "second again").unwrap();

    // create 3rd branch
    checkout_new_branch(&repo, &second_commit, "third-branch");
    commit_random_file(basedir.path(), &repo, "ipsum-iii", "third commit").unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&["push", "master"])
        .current_dir(&basedir.path())
        .assert()
        .success();
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&["push", "second-branch"])
        .current_dir(&basedir.path())
        .assert()
        .success();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("list")
        .current_dir(&basedir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "second-branch\nmaster\nthird-branch\n",
        ));
}
