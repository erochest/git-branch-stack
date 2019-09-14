mod utils;

use utils::*;

use git2::Repository;
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

    command(&basedir, &["push", "master"]);

    assert_branch(&repo, "master");
    assert_that(&basedir.path().join("ipsum-ii")).does_not_exist();
}
