use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

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
        let mut builder = WalkBuilder::new(&self.root);
        builder.hidden(true).git_ignore(true);

        let mut overrides = ignore::overrides::OverrideBuilder::new(&self.root);
        overrides.add("!node_modules").unwrap();
        overrides.add("!target").unwrap();
        overrides.add("!build").unwrap();
        overrides.add("!dist").unwrap();
        overrides.add("!coverage").unwrap();
        overrides.add("!reports").unwrap();
        overrides.add("!artifacts").unwrap();
        overrides.add("!scratch").unwrap();
        overrides.add("!.gemini").unwrap();

        let walker = builder.overrides(overrides.build().unwrap()).build();

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
