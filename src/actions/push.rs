use git2::build::CheckoutBuilder;
use git2::{BranchType, Repository};

use crate::errors::{BranchStackError, Result};
use crate::git::get_current_branch_name;
use crate::stack::FileStack;

pub fn push_branch(repo: &Repository, stack: &mut FileStack, branch_name: &String) -> Result<()> {
    let current_branch = get_current_branch_name(&repo)?;
    let branch = repo.find_branch(branch_name, BranchType::Local)?;
    let reference = branch.into_reference();
    let refname = reference
        .name()
        .ok_or_else(|| BranchStackError::InvalidBranchName(branch_name.clone()))?;

    repo.set_head(&refname)?;
    let mut checkout = CheckoutBuilder::default();
    // TODO: Make sure that this will `recreate_missing`.
    checkout.allow_conflicts(true);
    checkout.remove_untracked(true);
    repo.checkout_head(Some(&mut checkout))?;

    stack.push(current_branch);

    Ok(())
}
