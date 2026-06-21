use std::path::{Path, PathBuf};
use ignore::WalkBuilder;

pub struct RepositoryScanner {
    root: PathBuf,
}

impl RepositoryScanner {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn scan(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let walker = WalkBuilder::new(&self.root)
            .hidden(true)
            .git_ignore(true)
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        files.push(entry.into_path());
                    }
                }
                Err(_) => continue,
            }
        }
        files
    }
}
