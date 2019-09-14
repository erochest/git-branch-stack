/// # The File Stack
///
/// This is the core data type for the branch stack plugin. It's a stack
/// of strings that is persisted to a file on disc.
///
/// ```
/// # use tempfile::NamedTempFile;
/// # use spectral::prelude::*;
/// # use git_branch_stack::stack::FileStack;
///
/// let stack_file = NamedTempFile::new().unwrap();
///
/// {
///     let mut stack = FileStack::new(&stack_file.path()).unwrap();
///
///     stack.push("a".to_string());
///     stack.push("b".to_string());
///     stack.push("c".to_string());
///     assert_that(&stack.len()).is_equal_to(3);
///     assert_that(&stack.iter().collect::<Vec<&String>>()).is_equal_to(
///         &vec![&"c".to_string(), &"b".to_string(), &"a".to_string()]
///     );
///     assert_that(&stack.peek()).is_some().is_equal_to(&String::from("c"));
///     assert_that(&stack.pop()).is_some().is_equal_to(&String::from("c"));
///     assert_that(&stack.len()).is_equal_to(2);
///     // stack goes out of scope and writes the stack here.
/// }
///
/// {
///     // Let's open the stack file again.
///     let mut stack = FileStack::new(&stack_file.path()).unwrap();
///
///     assert_that(&stack.len()).is_equal_to(2);
///     assert_that(&stack.pop()).is_some().is_equal_to(&String::from("b"));
///     assert_that(&stack.pop()).is_some().is_equal_to(&String::from("a"));
/// }
/// ```
use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Write};
use std::iter::{IntoIterator, Iterator};
use std::path::{Path, PathBuf};

use crate::errors::{BranchStackError, Result};

/// The core FileStack struct.
#[derive(Debug)]
pub struct FileStack {
    filename: PathBuf,
    stack: VecDeque<String>,
}

impl FileStack {
    /// Creates a new FileStack given a file name. IO problems could raise
    /// an error.
    pub fn new<P: AsRef<Path>>(filename: &P) -> Result<FileStack> {
        let stack = FileStack::read_file(&filename)?;
        Ok(FileStack {
            filename: PathBuf::from(&filename.as_ref()),
            stack,
        })
    }

    /// The number of items in the stack.
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Add an item to the top of the stack.
    pub fn push(&mut self, item: String) {
        self.stack.push_front(item);
    }

    /// Remove an item from the top of the stack and return it.
    pub fn pop(&mut self) -> Option<String> {
        self.stack.pop_front()
    }

    /// What's on top of the stack?
    pub fn peek(&self) -> Option<String> {
        self.stack.front().cloned()
    }

    /// Move something buried to the top of the stack.
    ///
    /// This is analogous to `pushd` with a positive number.
    pub fn rotate_up(&mut self, n: usize) -> Result<()> {
        let n = n + 1;

        if n <= self.stack.len() {
            for _ in 0..n {
                if let Some(item) = self.stack.pop_back() {
                    self.stack.push_front(item);
                }
            }
            Ok(())
        } else {
            Err(BranchStackError::NoStackEntry)
        }
    }

    /// Move something buried to the bottom of the stack.
    ///
    /// This is analogous to `pushd` with a negative number.
    pub fn rotate_down(&mut self, n: usize) -> Result<()> {
        if n < self.stack.len() {
            for _ in 0..n {
                if let Some(item) = self.stack.pop_front() {
                    self.stack.push_back(item);
                }
            }
            Ok(())
        } else {
            Err(BranchStackError::NoStackEntry)
        }
    }

    /// Iterate over all of the items in the stack from top down.
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.stack.iter()
    }

    fn read_file<P: AsRef<Path>>(path: &P) -> Result<VecDeque<String>> {
        if !path.as_ref().exists() {
            Ok(VecDeque::new())
        } else {
            let mut file = File::open(&path)?;
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;
            Ok(buffer
                .lines()
                .map(|line: &str| line.trim().to_string())
                .collect())
        }
    }

    fn save(&self) -> Result<()> {
        let mut file = File::create(&self.filename)?;
        self.stack
            .iter()
            .try_for_each(|item| writeln!(file, "{}", item).map_err(BranchStackError::from))
    }
}

impl Drop for FileStack {
    fn drop(&mut self) {
        let _ = self.save();
    }
}

impl IntoIterator for FileStack {
    type Item = String;
    type IntoIter = ::std::collections::vec_deque::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let stack = self.stack.clone();
        stack.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    use assert_fs::fixture::NamedTempFile;
    use assert_fs::prelude::PathAssert;
    use spectral::prelude::*;
    use tempfile::tempdir;

    fn setup_stack(n: usize) -> (NamedTempFile, FileStack) {
        let stack_file = NamedTempFile::new("stack").unwrap();
        create_stack_file(
            &stack_file.path(),
            (0..n).map(|i| format!("{}", i)).collect(),
        );
        let stack = FileStack::new(&stack_file.path()).unwrap();
        (stack_file, stack)
    }

    fn create_stack_file<P: AsRef<Path>>(path: &P, items: Vec<String>) {
        let mut file = File::create(path).unwrap();
        for line in items {
            writeln!(file, "{}", line).unwrap();
        }
    }

    fn assert_stack(stack: FileStack, expected: Vec<&str>) {
        assert_that(&stack.into_iter().collect::<Vec<String>>()).is_equal_to(
            expected
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>(),
        );
    }

    #[test]
    fn new_should_initialize_empty_stack_from_missing_file() {
        let basedir = tempdir().unwrap();
        let stack_file = basedir.path().join("does-not-exist");
        let stack = FileStack::new(&stack_file);
        assert_that(&stack).is_ok();
        let stack = stack.unwrap();
        assert_that(&stack.len()).is_equal_to(0);
    }

    #[test]
    fn new_should_return_items_from_file() {
        let stack_file = NamedTempFile::new("stack").unwrap();
        create_stack_file(
            &stack_file.path(),
            vec!["one".to_string(), "two".to_string(), "three".to_string()],
        );
        let stack = FileStack::new(&stack_file.path()).unwrap();
        assert_that(&stack.len()).is_equal_to(3);
    }

    #[test]
    fn len_should_return_number_of_items() {
        let stack_file = NamedTempFile::new("stack").unwrap();
        create_stack_file(
            &stack_file.path(),
            (0..7).map(|i| format!("{}", i)).collect(),
        );
        let stack = FileStack::new(&stack_file.path()).unwrap();
        assert_that(&stack.len()).is_equal_to(7);
    }

    #[test]
    fn push_should_add_an_item_to_the_stack() {
        let stack_file = NamedTempFile::new("stack").unwrap();
        create_stack_file(
            &stack_file.path(),
            (0..7).map(|i| format!("{}", i)).collect(),
        );
        let mut stack = FileStack::new(&stack_file.path()).unwrap();
        assert_that(&stack.len()).is_equal_to(7);
        stack.push("hello".to_string());
        assert_that(&stack.len()).is_equal_to(8);
    }

    #[test]
    fn peek_returns_the_stack_top() {
        let stack_file = NamedTempFile::new("stack").unwrap();
        create_stack_file(
            &stack_file.path(),
            (0..7).map(|i| format!("{}", i)).collect(),
        );
        let mut stack = FileStack::new(&stack_file.path()).unwrap();
        assert_that(&stack.len()).is_equal_to(7);
        assert_that(&stack.peek())
            .is_some()
            .is_equal_to(&String::from("0"));
        stack.push("hello".to_string());
        assert_that(&stack.peek())
            .is_some()
            .is_equal_to(&String::from("hello"));
    }

    #[test]
    fn peek_on_empty_stack_returns_none() {
        let basedir = tempdir().unwrap();
        let stack_file = basedir.path().join("does-not-exist");
        let stack = FileStack::new(&stack_file);
        assert_that(&stack).is_ok();
        let stack = stack.unwrap();
        assert_that(&stack.len()).is_equal_to(0);
        assert_that(&stack.peek()).is_none();
    }

    #[test]
    fn pop_removes_items_from_stack() {
        let stack_file = NamedTempFile::new("stack").unwrap();
        create_stack_file(
            &stack_file.path(),
            (0..7).map(|i| format!("{}", i)).collect(),
        );
        let mut stack = FileStack::new(&stack_file.path()).unwrap();
        assert_that(&stack.len()).is_equal_to(7);
        assert_that(&stack.pop())
            .is_some()
            .is_equal_to(String::from("0"));
        assert_that(&stack.pop())
            .is_some()
            .is_equal_to(String::from("1"));
        assert_that(&stack.pop())
            .is_some()
            .is_equal_to(String::from("2"));
    }

    #[test]
    fn pop_on_empty_stack_returns_none() {
        let basedir = tempdir().unwrap();
        let stack_file = basedir.path().join("does-not-exist");
        let stack = FileStack::new(&stack_file);
        assert_that(&stack).is_ok();
        let mut stack = stack.unwrap();
        assert_that(&stack.len()).is_equal_to(0);
        assert_that(&stack.pop()).is_none();
    }

    #[test]
    fn drop_saves_changes_for_new_file() {
        let stack_file = NamedTempFile::new("stack").unwrap();
        {
            let mut stack = FileStack::new(&stack_file.path()).unwrap();
            stack.push("Ford".to_string());
            stack.push("Arthur".to_string());
            stack.push("Trillian".to_string());
        }
        stack_file.assert(
            r#"Trillian
Arthur
Ford
"#,
        );
    }

    #[test]
    fn into_iter_iterates_over_stack() {
        let stack_file = NamedTempFile::new("stack").unwrap();
        create_stack_file(
            &stack_file.path(),
            (0..7).map(|i| format!("{}", i)).collect(),
        );
        let stack = FileStack::new(&stack_file.path()).unwrap();
        assert_that(&stack.into_iter().collect::<Vec<String>>()).is_equal_to(
            &vec!["0", "1", "2", "3", "4", "5", "6"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>(),
        );
    }

    #[test]
    fn iter_iterates_over_stack() {
        let (_stack_file, stack) = setup_stack(7);
        assert_that(&stack.iter().collect::<Vec<&String>>()).is_equal_to(
            &vec![
                "0".to_string(),
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
            ]
            .iter()
            .collect::<Vec<&String>>(),
        );
    }

    #[test]
    fn rotate_up_0_raises_bottom_item() {
        let (_stack_file, mut stack) = setup_stack(4);
        assert_that(&stack.rotate_up(0)).is_ok();
        assert_stack(stack, vec!["3", "0", "1", "2"]);
    }

    #[test]
    fn rotate_up_1_raises_two_items() {
        let (_stack_file, mut stack) = setup_stack(4);
        assert_that(&stack.rotate_up(1)).is_ok();
        assert_stack(stack, vec!["2", "3", "0", "1"]);
    }

    #[test]
    fn rotate_up_2_raises_second_item() {
        let (_stack_file, mut stack) = setup_stack(4);
        assert_that(&stack.rotate_up(2)).is_ok();
        assert_stack(stack, vec!["1", "2", "3", "0"]);
    }

    #[test]
    fn rotate_up_3_maintains_stack() {
        let (_stack_file, mut stack) = setup_stack(4);
        assert_that(&stack.rotate_up(3)).is_ok();
        assert_stack(stack, vec!["0", "1", "2", "3"]);
    }

    #[test]
    fn rotate_up_4_returns_err() {
        let (_stack_file, mut stack) = setup_stack(4);
        assert_that(&stack.rotate_up(4))
            .is_err()
            .matches(|v| match v {
                BranchStackError::NoStackEntry => true,
                _ => false,
            });
    }

    #[test]
    fn rotate_down_0_maintains_stack() {
        let (_stack_file, mut stack) = setup_stack(4);
        assert_that(&stack.rotate_down(0)).is_ok();
        assert_stack(stack, vec!["0", "1", "2", "3"]);
    }

    #[test]
    fn rotate_down_1_moves_top_to_bottom() {
        let (_stack_file, mut stack) = setup_stack(4);
        assert_that(&stack.rotate_down(1)).is_ok();
        assert_stack(stack, vec!["1", "2", "3", "0"]);
    }

    #[test]
    fn rotate_down_2_moves_two_items() {
        let (_stack_file, mut stack) = setup_stack(4);
        assert_that(&stack.rotate_down(2)).is_ok();
        assert_stack(stack, vec!["2", "3", "0", "1"]);
    }

    #[test]
    fn rotate_down_3_moves_two_items() {
        let (_stack_file, mut stack) = setup_stack(4);
        assert_that(&stack.rotate_down(3)).is_ok();
        assert_stack(stack, vec!["3", "0", "1", "2"]);
    }

    #[test]
    fn rotate_down_4_moves_two_items() {
        let (_stack_file, mut stack) = setup_stack(4);
        assert_that(&stack.rotate_down(4))
            .is_err()
            .matches(|v| match v {
                BranchStackError::NoStackEntry => true,
                _ => false,
            });
    }
}
