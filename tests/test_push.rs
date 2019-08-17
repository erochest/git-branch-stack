use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use git2::{build::CheckoutBuilder, BranchType, Commit, Error, Repository};
use lipsum::lipsum;
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
    repo.branch("second-branch", &first_commit, false).unwrap();
    repo.set_head("refs/heads/second-branch").unwrap();
    let mut checkout = CheckoutBuilder::default();
    checkout.allow_conflicts(true);
    repo.checkout_head(Some(&mut checkout)).unwrap();
    commit_random_file(basedir.path(), &repo, "ipsum-ii", "second commit").unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&["push", "master"])
        .current_dir(&basedir.path())
        .assert()
        .success();

    // assert that we are on `master`
    let branch = repo.find_branch("master", BranchType::Local).unwrap();
    // Command::new("git")
    //     .arg("status")
    //     .current_dir(&basedir.path())
    //     .status()
    //     .unwrap();
    assert_that(&branch.is_head()).is_true();
    assert_that(&basedir.path().join("ipsum-ii")).does_not_exist();
}

fn make_initial_commit(repo: &Repository) {
    let author = repo.signature().unwrap();
    let mut index = repo.index().unwrap();
    let oid = index.write_tree().unwrap();
    let tree = repo.find_tree(oid).unwrap();
    repo.commit(Some("HEAD"), &author, &author, "initial commit", &tree, &[])
        .unwrap();
}

fn commit_random_file<'a>(
    dirname: &Path,
    repo: &'a Repository,
    filename: &str,
    commit_message: &str,
) -> Result<Commit<'a>, Error> {
    let author = repo.signature().unwrap();
    let mut index = repo.index().unwrap();
    let head = repo.refname_to_id("HEAD").unwrap();
    let head_commit = repo.find_commit(head).unwrap();

    // create content
    {
        let mut file = File::create(dirname.join(filename)).unwrap();
        writeln!(file, "{}", lipsum(75)).unwrap();
    }

    index.add_path(&Path::new(filename)).unwrap();
    let index_oid = index.write_tree().unwrap();
    let index_tree = repo.find_tree(index_oid).unwrap();

    let commit_id = repo
        .commit(
            Some("HEAD"),
            &author,
            &author,
            commit_message,
            &index_tree,
            &[&head_commit],
        )
        .unwrap();
    repo.find_commit(commit_id)
}
