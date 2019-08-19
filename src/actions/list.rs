use git2::Repository;

use crate::errors::Result;
use crate::git::get_current_branch_name;
use crate::stack::FileStack;

pub fn list_branch_stack(repo: &Repository, stack: &FileStack) -> Result<()> {
    let branch_name = get_current_branch_name(&repo)?;

    print!("{}", branch_name);
    for branch_name in stack.iter() {
        print!(" {}", branch_name);
    }
    println!();

    Ok(())
}
