use git2::{BranchType, Repository};

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