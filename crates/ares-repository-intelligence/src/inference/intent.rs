use crate::models::{GitEvidence, Timestamps};

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
    pub fn extract(commits: &[GitEvidence], ts: Option<&Timestamps>) -> ExtractedIntent {
        if commits.is_empty() && ts.is_none() {
            return ExtractedIntent {
                creation_reason: None,
                creation_hash: String::new(),
                creation_author: String::new(),
                evolution: Vec::new(),
                last_modified_hash: String::new(),
                last_modified_author: String::new(),
            };
        }

        let mut evolution = Vec::new();
        if !commits.is_empty() {
            let chronological: Vec<&GitEvidence> = commits.iter().rev().collect();
            evolution = chronological[1..]
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
        }

        // Use explicitly extracted introduction data if available, otherwise fallback to oldest known commit
        let (creation_reason, creation_hash, creation_author) = if let Some(t) = ts {
            if t.introduction_hash.is_some() {
                (
                    t.introduction_reason.clone(),
                    t.introduction_hash.clone().unwrap_or_default(),
                    t.introduced_by
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                )
            } else if !commits.is_empty() {
                let oldest = commits.iter().rev().next().unwrap();
                (
                    if oldest.message.is_empty() {
                        None
                    } else {
                        Some(oldest.message.clone())
                    },
                    oldest.hash[..7.min(oldest.hash.len())].to_string(),
                    if oldest.author.is_empty() {
                        "unknown".to_string()
                    } else {
                        oldest.author.clone()
                    },
                )
            } else {
                (None, String::new(), String::new())
            }
        } else if !commits.is_empty() {
            let oldest = commits.iter().rev().next().unwrap();
            (
                if oldest.message.is_empty() {
                    None
                } else {
                    Some(oldest.message.clone())
                },
                oldest.hash[..7.min(oldest.hash.len())].to_string(),
                if oldest.author.is_empty() {
                    "unknown".to_string()
                } else {
                    oldest.author.clone()
                },
            )
        } else {
            (None, String::new(), String::new())
        };

        let (last_modified_hash, last_modified_author) = if !commits.is_empty() {
            let newest = &commits[0];
            (
                newest.hash[..7.min(newest.hash.len())].to_string(),
                if newest.author.is_empty() {
                    "unknown".to_string()
                } else {
                    newest.author.clone()
                },
            )
        } else {
            (String::new(), String::new())
        };

        ExtractedIntent {
            creation_reason,
            creation_hash,
            creation_author,
            evolution,
            last_modified_hash,
            last_modified_author,
        }
    }
}
