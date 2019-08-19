mod utils;

use std::process::Command;

use utils::*;

use assert_cmd::prelude::*;
use git2::{BranchType, Repository};
use spectral::prelude::*;
use tempfile::tempdir;

#[test]
fn test_push() {
    let basedir = tempdir().unwrap();
    let repo = Repository::init(basedir.path()).unwrap();
    make_initial_commit(&repo);

    let first_commit =
        commit_random_file(basedir.path(), &repo, "ipsum-i", "first commit").unwrap();

    // create 2nd branch
    checkout_new_branch(&repo, &first_commit, "second-branch");
    commit_random_file(basedir.path(), &repo, "ipsum-ii", "second commit").unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&["push", "master"])
        .current_dir(&basedir.path())
        .assert()
        .success();

    // assert that we are on `master`
    let branch = repo.find_branch("master", BranchType::Local).unwrap();
    assert_that(&branch.is_head()).is_true();
    assert_that(&basedir.path().join("ipsum-ii")).does_not_exist();
}
