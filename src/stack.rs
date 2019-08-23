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
use std::fs::File;
use std::io::{Read, Write};
use std::iter::{IntoIterator, Iterator};
use std::path::{Path, PathBuf};

use crate::errors::Result;

/// The core FileStack struct.
#[derive(Debug)]
pub struct FileStack {
    filename: PathBuf,
    stack: Vec<String>,
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
        self.stack.push(item);
    }

    /// Remove an item from the top of the stack and return it.
    pub fn pop(&mut self) -> Option<String> {
        self.stack.pop()
    }

    /// What's on top of the stack?
    pub fn peek(&self) -> Option<String> {
        self.stack.last().cloned()
    }

    /// Iterate over all of the items in the stack from top down.
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.stack.iter().rev()
    }

    fn read_file<P: AsRef<Path>>(path: &P) -> Result<Vec<String>> {
        if !path.as_ref().exists() {
            Ok(Vec::new())
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
        let mut buffer = self.stack.join("\n");
        buffer += "\n";
        file.write_all(buffer.as_bytes())?;
        Ok(())
    }
}

impl Drop for FileStack {
    fn drop(&mut self) {
        let _ = self.save();
    }
}

impl IntoIterator for FileStack {
    type Item = String;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut stack = self.stack.clone();
        stack.reverse();
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

    fn create_stack_file<P: AsRef<Path>>(path: &P, items: Vec<String>) {
        let mut file = File::create(path).unwrap();
        for line in items {
            writeln!(file, "{}", line).unwrap();
        }
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
            .is_equal_to(&String::from("6"));
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
            .is_equal_to(String::from("6"));
        assert_that(&stack.pop())
            .is_some()
            .is_equal_to(String::from("5"));
        assert_that(&stack.pop())
            .is_some()
            .is_equal_to(String::from("4"));
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
            r#"Ford
Arthur
Trillian
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
            &vec!["6", "5", "4", "3", "2", "1", "0"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>(),
        );
    }

    #[test]
    fn iter_iterates_over_stack() {
        let stack_file = NamedTempFile::new("stack").unwrap();
        create_stack_file(
            &stack_file.path(),
            (0..7).map(|i| format!("{}", i)).collect(),
        );
        let stack = FileStack::new(&stack_file.path()).unwrap();
        assert_that(&stack.iter().collect::<Vec<&String>>()).is_equal_to(
            &vec![
                "6".to_string(),
                "5".to_string(),
                "4".to_string(),
                "3".to_string(),
                "2".to_string(),
                "1".to_string(),
                "0".to_string(),
            ]
            .iter()
            .collect::<Vec<&String>>(),
        );
    }
}
