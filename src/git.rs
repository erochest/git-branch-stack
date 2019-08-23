/// # git Utilities
///
/// These are a set of higher-level functions for common operations.
use git2::build::CheckoutBuilder;
use git2::{BranchType, ObjectType, Repository, ResetType};

use crate::errors::{BranchStackError, Result};

/// This returns the name of the current branch. If the user's not on a named
/// branch, this returns `Err(BranchStackError::NoCurrentBranch)`.
///
/// TODO: it may make more sense for this to return an `Option<String>` that
/// is `None` if the user's not on a named branch.
pub fn get_current_branch_name(repo: &Repository) -> Result<String> {
    let branch_name = repo
        .branches(Some(BranchType::Local))?
        .filter_map(|try_branch| try_branch.ok())
        .map(|pair| pair.0)
        .filter(|branch| branch.is_head())
        .filter_map(|branch| branch.name().ok().map(|b| b.map(String::from)))
        .filter_map(|branch_name| branch_name)
        .nth(0)
        .ok_or(BranchStackError::NoCurrrentBranch)?;
    Ok(branch_name)
}

/// Change to the branch named.
///
/// Currently this is implemented using `Repository.reset`. That's probably
/// not right.
pub fn change_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    let branch = repo.find_branch(branch_name, BranchType::Local)?;
    let reference = branch.get();
    let refname = reference
        .name()
        .ok_or_else(|| BranchStackError::InvalidBranchName(branch_name.to_string()))?;

    repo.set_head(&refname)?;

    let object = reference.peel(ObjectType::Commit)?;
    let mut checkout = CheckoutBuilder::default();
    repo.reset(&object, ResetType::Hard, Some(&mut checkout))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    use git2::build::CheckoutBuilder;
    use git2::{Commit, Error, ObjectType, Repository, ResetType, Signature};
    use lipsum::lipsum;
    use spectral::prelude::*;
    use tempfile::{tempdir, TempDir};

    use super::change_branch;

    #[test]
    fn test_change_branch_creates_missing_files() {
        let (working_dir, repo) = setup_repo();
        change_branch(&repo, "master").unwrap();
        let file2 = working_dir.path().join("file-2");
        assert_that(&file2).exists();
    }

    #[test]
    fn test_change_branch_leaves_untracked_files() {
        let (working_dir, repo) = setup_repo();
        let untracked = working_dir.path().join("untracked");
        random_file(&untracked);
        change_branch(&repo, "master").unwrap();
        assert_that(&untracked).exists();
    }

    #[test]
    fn test_change_branch_leaves_ignored_files() {
        let (working_dir, repo) = setup_repo();
        let ignored = working_dir.path().join("untracked-ignored");
        random_file(&ignored);
        let mut gitignore = File::create(working_dir.path().join(".gitignore")).unwrap();
        writeln!(gitignore, "untracked-ignored").unwrap();
        change_branch(&repo, "master").unwrap();
        assert_that(&ignored).exists();
    }

    #[test]
    fn test_change_branch_removes_tracked_files() {
        let (working_dir, repo) = setup_repo();
        change_branch(&repo, "master").unwrap();
        assert_that(&working_dir.path().join("file-3")).does_not_exist();
    }

    fn setup_repo() -> (TempDir, Repository) {
        let working_dir = tempdir().unwrap();
        let repo = Repository::init(working_dir.path()).unwrap();
        let sig = repo.signature().unwrap();

        {
            // initial commit
            let mut index = repo.index().unwrap();
            let oid = index.write_tree().unwrap();
            let tree = repo.find_tree(oid).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
                .unwrap();
        }

        {
            let commit =
                commit_random_file(working_dir.path(), &repo, &sig, "file-1", "commit 1").unwrap();
            commit_random_file(working_dir.path(), &repo, &sig, "file-2", "commit 2").unwrap();
            checkout_new_branch(&repo, &commit, "branch-2");
            commit_random_file(working_dir.path(), &repo, &sig, "file-3", "commit 3").unwrap();
        }

        (working_dir, repo)
    }

    fn random_file<P: AsRef<Path>>(path: P) {
        let mut file = File::create(path.as_ref()).unwrap();
        writeln!(file, "{}", lipsum(75)).unwrap();
    }

    fn commit_random_file<'a>(
        dirname: &Path,
        repo: &'a Repository,
        author: &Signature<'a>,
        filename: &str,
        commit_message: &str,
    ) -> Result<Commit<'a>, Error> {
        let mut index = repo.index().unwrap();
        let head = repo.refname_to_id("HEAD").unwrap();
        let head_commit = repo.find_commit(head).unwrap();

        random_file(dirname.join(filename));

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

    fn checkout_new_branch<'a>(repo: &Repository, commit: &Commit<'a>, branch_name: &str) {
        repo.branch(branch_name, &commit, false).unwrap();
        let refname = format!("refs/heads/{}", branch_name);
        repo.set_head(&refname).unwrap();

        let mut checkout = CheckoutBuilder::default();
        checkout.allow_conflicts(true);
        checkout.remove_untracked(true);
        repo.checkout_head(Some(&mut checkout)).unwrap();
    }
}
