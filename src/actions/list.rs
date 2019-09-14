/// # List Command
///
/// This executes the `list` command. It prints the current branch name as
/// well as the stack.
use git2::Repository;

use crate::errors::Result;
use crate::git::get_current_branch_name;
use crate::stack::FileStack;

pub fn list_branch_stack(repo: &Repository, stack: &FileStack) -> Result<()> {
    let branch_name = get_current_branch_name(&repo)?;

    println!("{}", branch_name);
    for branch_name in stack.iter() {
        println!("{}", branch_name);
    }

    Ok(())
}
