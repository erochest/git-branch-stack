mod utils;

use git_branch_stack::git::{change_branch, get_current_branch_name};
use utils::*;

use git2::Repository;
use tempfile::tempdir;

#[test]
fn test_rotate() {
    let basedir = tempdir().unwrap();
    let repo = Repository::init(basedir.path()).unwrap();
    make_initial_commit(&repo);

    let first_commit =
        commit_random_file(basedir.path(), &repo, "ipsum-i", "first commit").unwrap();

    // create 2nd branch
    checkout_new_branch(&repo, &first_commit, "second-branch");
    commit_random_file(basedir.path(), &repo, "ipsum-ii", "second commit").unwrap();
    // create 3rd branch
    checkout_new_branch(&repo, &first_commit, "third-branch");
    commit_random_file(basedir.path(), &repo, "ipsum-iii", "third commit").unwrap();

    change_branch(&repo, "master").unwrap();
    command(&basedir, &["push", "second-branch"]);
    command(&basedir, &["push", "third-branch"]);

    // +0 raises bottom
    command(&basedir, &["push", "+0"]);
    assert_branch(&repo, "master");
    command(&basedir, &["list"]).stdout("master\nthird-branch\nsecond-branch\n");

    // -0 no change
    command(&basedir, &["push", "--", "-0"]);
    assert_branch(&repo, "master");
    command(&basedir, &["list"]).stdout("master\nthird-branch\nsecond-branch\n");

    // +1 raises two
    command(&basedir, &["push", "+1"]);
    assert_branch(&repo, "third-branch");
    command(&basedir, &["list"]).stdout("third-branch\nsecond-branch\nmaster\n");

    // -1 raises bottom
    command(&basedir, &["push", "--", "-1"]);
    eprintln!(
        "current branch: {:?}",
        get_current_branch_name(&repo).unwrap()
    );
    assert_branch(&repo, "second-branch");
    command(&basedir, &["list"]).stdout("second-branch\nmaster\nthird-branch\n");
}
