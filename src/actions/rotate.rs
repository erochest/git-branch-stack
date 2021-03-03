use git2::Repository;

use crate::errors::{BranchStackError, Result};
use crate::git::{change_branch, get_current_branch_name};
use crate::stack::FileStack;

#[derive(Debug, Eq, PartialEq)]
pub enum RotateDirection {
    Up,
    Down,
}

pub fn rotate_branch(
    repo: &Repository,
    stack: &mut FileStack,
    dir: RotateDirection,
    n: usize,
) -> Result<()> {
    let current_branch = get_current_branch_name(repo)?;
    stack.push(current_branch);

    // eprintln!("pre-rotate: {:?}", stack);
    match dir {
        RotateDirection::Up => stack.rotate_up(n)?,
        RotateDirection::Down => stack.rotate_down(n)?,
    }
    // eprintln!("post-rotate: {:?}", stack);

    let new_branch = stack.pop().ok_or(BranchStackError::NoStackEntry)?;
    println!("{}", new_branch);
    change_branch(repo, &new_branch)
}

pub fn parse_rotation(input: &str) -> Option<(RotateDirection, usize)> {
    let direction = input.get(0..1).and_then(|prefix| match prefix {
        "+" => Some(RotateDirection::Up),
        "-" => Some(RotateDirection::Down),
        _ => None,
    })?;
    input
        .get(1..)
        .and_then(|n_str| n_str.parse().ok())
        .map(|n| (direction, n))
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;

    use super::{parse_rotation, RotateDirection};

    #[test]
    fn test_parse_rotation_returns_none_on_branch() {
        assert_that(&parse_rotation("master")).is_none();
    }

    #[test]
    fn test_parse_rotation_parses_positive_number() {
        assert_that(&parse_rotation("+0"))
            .is_some()
            .is_equal_to((RotateDirection::Up, 0));
    }

    #[test]
    fn test_parse_rotation_parses_negative_number() {
        assert_that(&parse_rotation("-34"))
            .is_some()
            .is_equal_to((RotateDirection::Down, 34));
    }

    #[test]
    fn test_parse_rotation_parses_negative_zero() {
        assert_that(&parse_rotation("-0"))
            .is_some()
            .is_equal_to((RotateDirection::Down, 0));
    }
}
