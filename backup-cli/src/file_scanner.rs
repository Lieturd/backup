use std::collections::HashSet;
use std::fs;
use std::io::Error as IoError;
use std::path::PathBuf;

use walkdir::{WalkDir, IntoIter as WalkerIter};

#[derive(Debug)]
pub struct FileScanner<I> {
    dir_iter: I,
    exclude: HashSet<PathBuf>,
    current_walker: Option<WalkerIter>,
}

impl<I> FileScanner<I>
    where I: Iterator<Item = PathBuf>,
{
    pub fn new<J, E>(dir_iter: J, exclude_iter: E) -> Result<FileScanner<I>, IoError>
        where J: IntoIterator<IntoIter = I, Item = PathBuf>,
              E: IntoIterator<Item = PathBuf>,
    {
        let mut exclude = HashSet::new();
        // Enter canonical paths into exclude set.
        for exclude_path in exclude_iter {
            // Errors and returns if we see a bad path.
            // Perhaps we should just log it?
            exclude.insert(fs::canonicalize(exclude_path)?);
        }

        Ok(FileScanner {
            dir_iter: dir_iter.into_iter(),
            exclude: exclude,
            current_walker: None,
        })
    }
}

impl<I> Iterator for FileScanner<I>
    where I: Iterator<Item = PathBuf>,
{
    type Item = Result<PathBuf, IoError>;

    fn next(&mut self) -> Option<Self::Item> {
        // If we have a walker, continue iterating on it.
        if let Some(ref mut walker) = self.current_walker {
            match walker.next() {
                Some(Ok(entry)) => {
                    // If we have a proper entry, canonicalize and check if it's excluded.
                    let canonical = match fs::canonicalize(entry.path()) {
                        Ok(path) => path,
                        // If the path can't be canonicalized, pass up the error.
                        Err(e) => return Some(Err(e)),
                    };
                    if self.exclude.contains(&canonical) {
                        // If it's excluded, skip it and recurse.
                        walker.skip_current_dir();
                        self.next()
                    }
                    else {
                        // If it's not excluded, return it.
                        Some(Ok(canonical))
                    }
                }
                Some(Err(e)) => {
                    // If we got an error, pass it up.
                    Some(Err(e.into()))
                }
                None => {
                    // If the walker is exhausted, clear it and recurse.
                    self.current_walker = None;
                    self.next()
                }
            }
        }
        else {
            // If we don't have a walker, go grab the next directory to search.
            if let Some(directory) = self.dir_iter.next() {
                // Make a new walker and recurse.
                self.current_walker = Some(WalkDir::new(directory).into_iter());
                self.next()
            }
            else {
                // No more directories means we're done. Return None.
                None
            }
        }
    }
}
