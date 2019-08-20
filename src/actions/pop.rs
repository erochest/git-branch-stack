use git2::Repository;

use crate::errors::{BranchStackError, Result};
use crate::git::change_branch;
use crate::stack::FileStack;

pub fn pop_branch_stack(repo: &Repository, stack: &mut FileStack) -> Result<()> {
    stack
        .pop()
        .ok_or(BranchStackError::EmptyStack)
        .and_then(|branch_name| {
            println!("{}", branch_name);
            change_branch(repo, &branch_name)
        })
}
