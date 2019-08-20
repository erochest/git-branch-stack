use git2::build::CheckoutBuilder;
use git2::{BranchType, Repository};

use crate::errors::{BranchStackError, Result};
use crate::git::{change_branch, get_current_branch_name};
use crate::stack::FileStack;

pub fn push_branch(repo: &Repository, stack: &mut FileStack, branch_name: &String) -> Result<()> {
    let current_branch = get_current_branch_name(&repo)?;
    change_branch(&repo, branch_name)?;
    stack.push(current_branch);
    Ok(())
}
