use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub enum RetryPolicy {
    Immediate {
        max_retries: i32,
    },
    FixedDelay {
        max_retries: i32,
        delay_ms: u64,
    },
    ExponentialBackoff {
        max_retries: i32,
        initial_delay_ms: u64,
        multiplier: f32,
    },
}

impl RetryPolicy {
    pub fn calculate_delay_ms(&self, attempt: i32) -> Option<u64> {
        match self {
            Self::Immediate { max_retries } => {
                if attempt < *max_retries {
                    Some(0)
                } else {
                    None
                }
            }
            Self::FixedDelay {
                max_retries,
                delay_ms,
            } => {
                if attempt < *max_retries {
                    Some(*delay_ms)
                } else {
                    None
                }
            }
            Self::ExponentialBackoff {
                max_retries,
                initial_delay_ms,
                multiplier,
            } => {
                if attempt < *max_retries {
                    Some((*initial_delay_ms as f32 * multiplier.powi(attempt)).round() as u64)
                } else {
                    None
                }
            }
        }
    }
}
