use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct FileArtifact {
    pub path: String,
    pub content: String,
}

pub struct ArtifactParser;

impl ArtifactParser {
    /// Parses deterministic file blocks formatted as:
    /// ```file:path=src/main.rs
    /// fn main() {}
    /// ```
    pub fn parse(text: &str) -> Vec<FileArtifact> {
        let mut artifacts = Vec::new();
        // Regex to capture the path and the content inside the code block
        // We use (?s) for dot to match newline
        let re = Regex::new(r"(?s)```file:path=([^\r\n]+)\r?\n(.*?)\r?\n```").unwrap();

        for cap in re.captures_iter(text) {
            let path = cap.get(1).unwrap().as_str().trim().to_string();
            let content = cap.get(2).unwrap().as_str().to_string();
            artifacts.push(FileArtifact { path, content });
        }

        artifacts
    }
}

pub struct ArtifactWriter;

impl ArtifactWriter {
    /// Writes the parsed artifacts to the specified workspace directory
    pub fn write_all(workspace_dir: &Path, artifacts: &[FileArtifact]) -> Result<()> {
        for artifact in artifacts {
            let full_path = workspace_dir.join(&artifact.path);

            // Ensure parent directories exist
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Write content
            fs::write(full_path, &artifact.content)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_artifacts() {
        let text = r#"
Here is the code:
```file:path=src/main.rs
fn main() {
    println!("Hello, ARES!");
}
```
And another file:
```file:path=Cargo.toml
[package]
name = "test"
```
"#;
        let artifacts = ArtifactParser::parse(text);
        assert_eq!(artifacts.len(), 2);
        assert_eq!(artifacts[0].path, "src/main.rs");
        assert!(artifacts[0].content.contains("println!"));
        assert_eq!(artifacts[1].path, "Cargo.toml");
        assert!(artifacts[1].content.contains("name = \"test\""));
    }
}
