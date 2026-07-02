use crate::models::GitEvidence;

#[derive(Debug, Clone)]
pub struct ExtractedIntent {
    pub creation_reason: Option<String>,
    pub creation_hash: String,
    pub creation_author: String,
    pub evolution: Vec<EvolutionStep>,
    pub last_modified_hash: String,
    pub last_modified_author: String,
}

#[derive(Debug, Clone)]
pub struct EvolutionStep {
    pub description: String,
    pub hash: String,
    pub author: String,
}

pub struct IntentExtractor;

impl IntentExtractor {
    pub fn extract(commits: &[GitEvidence]) -> ExtractedIntent {
        if commits.is_empty() {
            return ExtractedIntent {
                creation_reason: None,
                creation_hash: String::new(),
                creation_author: String::new(),
                evolution: Vec::new(),
                last_modified_hash: String::new(),
                last_modified_author: String::new(),
            };
        }

        // commits are sorted most-recent-first; reverse for chronological
        let chronological: Vec<&GitEvidence> = commits.iter().rev().collect();
        let oldest = chronological[0];
        let newest = &commits[0];

        // Evolution: everything after creation, in chronological order
        let evolution: Vec<EvolutionStep> = chronological[1..]
            .iter()
            .map(|c| EvolutionStep {
                description: if c.message.is_empty() {
                    "(no message)".to_string()
                } else {
                    c.message.clone()
                },
                hash: c.hash[..7.min(c.hash.len())].to_string(),
                author: if c.author.is_empty() {
                    "unknown".to_string()
                } else {
                    c.author.clone()
                },
            })
            .collect();

        ExtractedIntent {
            creation_reason: if oldest.message.is_empty() {
                None
            } else {
                Some(oldest.message.clone())
            },
            creation_hash: oldest.hash[..7.min(oldest.hash.len())].to_string(),
            creation_author: if oldest.author.is_empty() {
                "unknown".to_string()
            } else {
                oldest.author.clone()
            },
            evolution,
            last_modified_hash: newest.hash[..7.min(newest.hash.len())].to_string(),
            last_modified_author: if newest.author.is_empty() {
                "unknown".to_string()
            } else {
                newest.author.clone()
            },
        }
    }
}
