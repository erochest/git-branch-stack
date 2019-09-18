mod utils;

use std::process::Command;

use utils::*;

use assert_cmd::prelude::*;
use git2::{BranchType, Repository};
use predicates::prelude::*;
use spectral::prelude::*;
use tempfile::tempdir;

#[test]
fn test_pop() {
    let basedir = tempdir().unwrap();
    let basepath = basedir.path();
    let repo = Repository::init(basepath).unwrap();
    make_initial_commit(&repo);

    let first_commit = commit_random_file(basepath, &repo, "ipsum-i", "first commit").unwrap();

    // 2nd branch
    checkout_new_branch(&repo, &first_commit, "second-branch");
    commit_random_file(basepath, &repo, "ipsum-ii", "second commit").unwrap();
    let second_commit = commit_random_file(basepath, &repo, "ipsum-ii-a", "second again").unwrap();

    // 3rd branch
    checkout_new_branch(&repo, &second_commit, "third-branch");
    commit_random_file(basepath, &repo, "ipsum-iii", "third commit").unwrap();

    command(basepath, &["push", "master"]);
    command(basepath, &["push", "second-branch"]);

    command(basepath, &["list"])
        .stdout(predicate::str::contains(
            "second-branch\nmaster\nthird-branch\n",
        ));

    command(basepath, &["pop"]);

    command(basepath, &["list"])
        .stdout(predicate::str::contains("master\nthird-branch\n"));

    // assert that we are on `master`
    let branch = repo.find_branch("master", BranchType::Local).unwrap();
    assert_that(&branch.is_head()).is_true();
    assert_that(&basedir.path().join("ipsum-ii")).does_not_exist();
    assert_that(&basedir.path().join("ipsum-iii")).does_not_exist();

    // pop master
    command(basepath, &["pop"]);
    // pop third-branch (empty stack)
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("pop")
        .current_dir(basepath)
        .assert()
        .failure()
        .stderr(predicate::str::contains("EmptyStack"));
}
