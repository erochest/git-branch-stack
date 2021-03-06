use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use assert_cmd::assert::Assert;
use assert_cmd::prelude::*;
use git2::build::CheckoutBuilder;
use git2::{BranchType, Commit, Error, ObjectType, Repository, ResetType, Signature};
use lipsum::lipsum;
use spectral::prelude::*;

pub fn make_initial_commit(repo: &Repository) {
    let author = Signature::now("Trillian McMillan", "tmcmilla@heartofgold.ship").unwrap();
    let mut index = repo.index().unwrap();
    let oid = index.write_tree().unwrap();
    let tree = repo.find_tree(oid).unwrap();
    repo.commit(Some("HEAD"), &author, &author, "initial commit", &tree, &[])
        .unwrap();
}

pub fn commit_random_file<'a>(
    dirname: &Path,
    repo: &'a Repository,
    filename: &str,
    commit_message: &str,
) -> Result<Commit<'a>, Error> {
    let author = Signature::now("Trillian McMillan", "tmcmilla@heartofgold.ship").unwrap();
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

    let object = repo
        .find_object(commit_id, Some(ObjectType::Commit))
        .unwrap();
    let mut checkout = CheckoutBuilder::default();
    repo.reset(&object, ResetType::Hard, Some(&mut checkout))
        .unwrap();
    repo.find_commit(commit_id)
}

pub fn checkout_new_branch<'a>(repo: &Repository, commit: &Commit<'a>, branch_name: &str) {
    repo.branch(branch_name, &commit, false).unwrap();
    let refname = format!("refs/heads/{}", branch_name);
    repo.set_head(&refname).unwrap();

    let mut checkout = CheckoutBuilder::default();
    checkout.allow_conflicts(true);
    checkout.remove_untracked(true);
    repo.checkout_head(Some(&mut checkout)).unwrap();
}

pub fn _status<P: AsRef<Path>>(path: P, message: Option<&str>) {
    message.into_iter().for_each(|m| println!("XXX {}", m));
    Command::new("ls")
        .current_dir(path.as_ref())
        .status()
        .unwrap();
    Command::new("git")
        .arg("status")
        .current_dir(path.as_ref())
        .status()
        .unwrap();
}

pub fn command<P: AsRef<Path>>(path: P, args: &[&str]) -> Assert {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(args)
        .current_dir(path.as_ref())
        .assert()
        .success()
}

pub fn assert_branch(repo: &Repository, branch_name: &str) {
    let branch = repo.find_branch(branch_name, BranchType::Local).unwrap();
    assert_that(&branch.is_head()).is_true();
}
