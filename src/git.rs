use git2::build::CheckoutBuilder;
use git2::{BranchType, ObjectType, Repository, ResetType};

use crate::errors::{BranchStackError, Result};

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
