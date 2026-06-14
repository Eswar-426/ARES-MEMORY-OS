use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ModelState {
    Unknown,
    Healthy,
    Degraded,
    RateLimited,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRecord {
    pub state: ModelState,
    pub last_failure: Option<DateTime<Utc>>,
    pub failure_count: u32,
}

impl Default for HealthRecord {
    fn default() -> Self {
        Self {
            state: ModelState::Unknown,
            last_failure: None,
            failure_count: 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelConfig {
    pub provider: String,
    pub id: String,
    pub priority: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChainConfig {
    pub architecture: Vec<ModelConfig>,
    pub feature: Vec<ModelConfig>,
    pub debug: Vec<ModelConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelsYaml {
    pub chains: HashMap<String, ChainConfig>,
}

pub struct ModelCatalog {
    pub config: ModelsYaml,
}

impl ModelCatalog {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ModelsYaml = serde_yaml::from_str(&content)?;
        Ok(Self { config })
    }

    pub fn get_models_for_role(&self, chain: &str, role: &str) -> Option<Vec<ModelConfig>> {
        let chain_cfg = self.config.chains.get(chain)?;
        let mut models = match role {
            "architecture" => chain_cfg.architecture.clone(),
            "feature" => chain_cfg.feature.clone(),
            "debug" => chain_cfg.debug.clone(),
            _ => return None,
        };
        // Sort by priority ascending
        models.sort_by_key(|m| m.priority);
        Some(models)
    }
}

pub struct ModelHealthChecker {
    health_map: HashMap<String, HealthRecord>,
}

impl ModelHealthChecker {
    pub fn new() -> Self {
        Self {
            health_map: HashMap::new(),
        }
    }
}

impl Default for ModelHealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelHealthChecker {
    pub fn get_state(&self, model_id: &str) -> ModelState {
        self.health_map
            .get(model_id)
            .map(|r| r.state)
            .unwrap_or(ModelState::Unknown)
    }

    pub fn get_health_map(&self) -> &HashMap<String, HealthRecord> {
        &self.health_map
    }

    pub fn is_available(&mut self, model_id: &str) -> bool {
        let record = self.health_map.entry(model_id.to_string()).or_default();

        // Cooldown logic: if Degraded or RateLimited, check if 30 mins have passed
        match record.state {
            ModelState::Disabled => false,
            ModelState::Degraded | ModelState::RateLimited => {
                if let Some(last) = record.last_failure {
                    let now = Utc::now();
                    if now.signed_duration_since(last).num_minutes() > 30 {
                        record.state = ModelState::Unknown; // Ready to retry
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => true,
        }
    }

    pub fn mark_success(&mut self, model_id: &str) {
        let record = self.health_map.entry(model_id.to_string()).or_default();
        record.state = ModelState::Healthy;
        record.failure_count = 0;
    }

    pub fn mark_failure(&mut self, model_id: &str, error_msg: &str) {
        let record = self.health_map.entry(model_id.to_string()).or_default();
        record.last_failure = Some(Utc::now());
        record.failure_count += 1;

        if error_msg.contains("402") || error_msg.contains("429") {
            record.state = ModelState::RateLimited;
        } else if error_msg.contains("404") {
            record.state = ModelState::Disabled; // 404 means endpoint dead/invalid
        } else {
            record.state = ModelState::Degraded;
        }
    }
}

pub struct ProviderRegistry {
    catalog: ModelCatalog,
    health_checker: ModelHealthChecker,
    openrouter_key: Option<String>,
    gemini_key: Option<String>,
    nvidia_key: Option<String>,
    groq_key: Option<String>,
    active_chain: String,
    dynamic_chains: HashMap<String, Vec<ModelConfig>>,
}

#[derive(Debug, Clone)]
pub struct ModelCapability {
    pub provider: String,
    pub id: String,
    pub score_architecture: u32,
    pub score_feature: u32,
    pub score_debug: u32,
}

pub struct DiscoveryEngine {
    capabilities: Vec<ModelCapability>,
    pub available_models: Vec<(String, String)>, // (provider, id)
}

impl DiscoveryEngine {
    pub fn new() -> Self {
        Self {
            capabilities: vec![
                ModelCapability {
                    provider: "nvidia".to_string(),
                    id: "meta/llama-3.3-70b-instruct".to_string(),
                    score_architecture: 100,
                    score_feature: 90,
                    score_debug: 85,
                },
                ModelCapability {
                    provider: "groq".to_string(),
                    id: "deepseek-r1-distill-llama-70b".to_string(),
                    score_architecture: 85,
                    score_feature: 95,
                    score_debug: 100,
                },
                ModelCapability {
                    provider: "openrouter".to_string(),
                    id: "meta-llama/llama-3.3-70b-instruct:free".to_string(),
                    score_architecture: 95,
                    score_feature: 90,
                    score_debug: 85,
                },
                ModelCapability {
                    provider: "openrouter".to_string(),
                    id: "google/gemma-3-27b-it:free".to_string(),
                    score_architecture: 70,
                    score_feature: 80,
                    score_debug: 70,
                },
            ],
            available_models: vec![],
        }
    }
}

impl Default for DiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoveryEngine {
    pub fn build_chain_for_role(&self, role: &str) -> Vec<ModelConfig> {
        let mut scored: Vec<(u32, ModelConfig)> = vec![];

        for (provider, id) in &self.available_models {
            if let Some(cap) = self
                .capabilities
                .iter()
                .find(|c| c.provider == *provider && c.id == *id)
            {
                let score = match role {
                    "architecture" => cap.score_architecture,
                    "feature" => cap.score_feature,
                    "debug" => cap.score_debug,
                    _ => 0,
                };
                if score > 0 {
                    scored.push((
                        score,
                        ModelConfig {
                            provider: provider.clone(),
                            id: id.clone(),
                            priority: 0, // Will assign later
                        },
                    ));
                }
            }
        }

        // Sort descending by score
        scored.sort_by_key(|b| std::cmp::Reverse(b.0));

        let mut chain = vec![];
        for (i, (_, mut cfg)) in scored.into_iter().enumerate() {
            cfg.priority = (i + 1) as u32;
            chain.push(cfg);
        }
        chain
    }
}

impl ProviderRegistry {
    pub fn new(
        catalog: ModelCatalog,
        openrouter_key: Option<String>,
        gemini_key: Option<String>,
        nvidia_key: Option<String>,
        groq_key: Option<String>,
        active_chain: String,
    ) -> Self {
        Self {
            catalog,
            health_checker: ModelHealthChecker::new(),
            openrouter_key,
            gemini_key,
            nvidia_key,
            groq_key,
            active_chain,
            dynamic_chains: HashMap::new(),
        }
    }

    pub async fn build_dynamic_chains(&mut self) {
        println!("🔍 Discovering Available Models...");
        let mut discovery = DiscoveryEngine::new();
        let client = reqwest::Client::new();

        // Discover NVIDIA
        if let Some(key) = &self.nvidia_key {
            let res = client
                .get("https://integrate.api.nvidia.com/v1/models")
                .header("Authorization", format!("Bearer {}", key))
                .header("Accept", "application/json")
                .send()
                .await;
            if let Ok(r) = res {
                if r.status().is_success() {
                    println!("    ✅ Found NVIDIA API");
                    if let Ok(json) = r.json::<serde_json::Value>().await {
                        if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                            for item in data {
                                if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                                    discovery
                                        .available_models
                                        .push(("nvidia".to_string(), id.to_string()));
                                }
                            }
                        }
                    }
                } else {
                    println!("    ❌ NVIDIA API failed to list models");
                }
            }
        }

        // Discover Groq
        if let Some(key) = &self.groq_key {
            let res = client
                .get("https://api.groq.com/openai/v1/models")
                .header("Authorization", format!("Bearer {}", key))
                .send()
                .await;
            if let Ok(r) = res {
                if r.status().is_success() {
                    println!("    ✅ Found Groq API");
                    if let Ok(json) = r.json::<serde_json::Value>().await {
                        if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                            for item in data {
                                if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                                    discovery
                                        .available_models
                                        .push(("groq".to_string(), id.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Add known OpenRouter fallback models if key is present (OR listing is too huge to match everything simply)
        if self.openrouter_key.is_some() {
            println!("    ✅ Found OpenRouter API");
            discovery.available_models.push((
                "openrouter".to_string(),
                "meta-llama/llama-3.3-70b-instruct:free".to_string(),
            ));
            discovery.available_models.push((
                "openrouter".to_string(),
                "google/gemma-3-27b-it:free".to_string(),
            ));
        }

        // Build chains
        self.dynamic_chains.insert(
            "architecture".to_string(),
            discovery.build_chain_for_role("architecture"),
        );
        self.dynamic_chains.insert(
            "feature".to_string(),
            discovery.build_chain_for_role("feature"),
        );
        self.dynamic_chains
            .insert("debug".to_string(), discovery.build_chain_for_role("debug"));

        println!("🚀 Dynamic Chains Assembled:");
        for role in &["architecture", "feature", "debug"] {
            if let Some(chain) = self.dynamic_chains.get(*role) {
                println!("   - {}: {} models", role, chain.len());
            }
        }
        println!();
    }

    pub fn get_dynamic_chains(&self) -> &HashMap<String, Vec<ModelConfig>> {
        &self.dynamic_chains
    }

    pub fn get_health_map(&self) -> &HashMap<String, HealthRecord> {
        self.health_checker.get_health_map()
    }

    pub async fn health_check(&mut self) {
        println!("🩺 Running Provider Health Checks...");
        let client = reqwest::Client::new();

        // Check OpenRouter
        if let Some(key) = &self.openrouter_key {
            let res = client
                .get("https://openrouter.ai/api/v1/models")
                .header("Authorization", format!("Bearer {}", key))
                .send()
                .await;
            if let Ok(r) = res {
                if r.status().is_success() {
                    println!("    ✅ OpenRouter: OK");
                } else {
                    println!("    ❌ OpenRouter: Failed ({})", r.status());
                }
            } else {
                println!("    ❌ OpenRouter: Network Error");
            }
        } else {
            println!("    ⏭️  OpenRouter: No API Key provided");
        }

        // Check Gemini
        if let Some(key) = &self.gemini_key {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                key
            );
            let res = client.get(&url).send().await;
            if let Ok(r) = res {
                if r.status().is_success() {
                    println!("    ✅ Gemini: OK");
                } else {
                    println!("    ❌ Gemini: Failed ({})", r.status());
                }
            } else {
                println!("    ❌ Gemini: Network Error");
            }
        } else {
            println!("    ⏭️  Gemini: No API Key provided");
        }

        // Check NVIDIA
        if let Some(key) = &self.nvidia_key {
            let res = client
                .get("https://integrate.api.nvidia.com/v1/models")
                .header("Authorization", format!("Bearer {}", key))
                .header("Accept", "application/json")
                .send()
                .await;
            if let Ok(r) = res {
                if r.status().is_success() {
                    println!("    ✅ NVIDIA NIM: OK");
                } else {
                    println!("    ❌ NVIDIA NIM: Failed ({})", r.status());
                }
            } else {
                println!("    ❌ NVIDIA NIM: Network Error");
            }
        } else {
            println!("    ⏭️  NVIDIA NIM: No API Key provided");
        }

        // Check Groq
        if let Some(key) = &self.groq_key {
            let res = client
                .get("https://api.groq.com/openai/v1/models")
                .header("Authorization", format!("Bearer {}", key))
                .send()
                .await;
            if let Ok(r) = res {
                if r.status().is_success() {
                    println!("    ✅ Groq: OK");
                } else {
                    println!("    ❌ Groq: Failed ({})", r.status());
                }
            } else {
                println!("    ❌ Groq: Network Error");
            }
        } else {
            println!("    ⏭️  Groq: No API Key provided");
        }
        println!();
    }

    pub async fn ask_with_fallback(
        &mut self,
        role: &str,
        context: &str,
        prompt: &str,
    ) -> Result<(String, String, usize)> {
        // Try dynamic chain first, fallback to catalog if empty or missing
        let empty_vec = vec![];
        let dynamic = self.dynamic_chains.get(role).unwrap_or(&empty_vec);
        let models = if !dynamic.is_empty() {
            dynamic.clone()
        } else {
            self.catalog
                .get_models_for_role(&self.active_chain, role)
                .ok_or_else(|| anyhow!("Role {} not found in active chain", role))?
        };

        let mut fallback_count = 0;

        for model_cfg in models {
            let model_id = &model_cfg.id;

            if !self.health_checker.is_available(model_id) {
                println!(
                    "    ⏩ Skipping {} (Currently {:?})",
                    model_id,
                    self.health_checker.get_state(model_id)
                );
                continue;
            }

            println!("    🤖 Provider: {} (via {})", model_id, model_cfg.provider);

            let provider: Box<dyn crate::providers::r#trait::ModelProvider> =
                match model_cfg.provider.as_str() {
                    "openrouter" => {
                        let key = self.openrouter_key.clone().unwrap_or_default();
                        Box::new(crate::providers::openrouter::OpenRouterProvider::new(
                            key, model_id,
                        ))
                    }
                    "gemini" => {
                        let key = self.gemini_key.clone().unwrap_or_default();
                        Box::new(crate::providers::gemini::GeminiProvider::new(key, model_id))
                    }
                    "nvidia" => {
                        let key = self.nvidia_key.clone().unwrap_or_default();
                        Box::new(crate::providers::nvidia_nim::NvidiaNimProvider::new(
                            key, model_id,
                        ))
                    }
                    "groq" => {
                        let key = self.groq_key.clone().unwrap_or_default();
                        Box::new(crate::providers::groq::GroqProvider::new(key, model_id))
                    }
                    _ => {
                        println!("    ❌ Unknown provider: {}", model_cfg.provider);
                        continue;
                    }
                };

            match provider.ask(context, prompt).await {
                Ok(response) => {
                    self.health_checker.mark_success(model_id);
                    println!("    ✅ Success");
                    return Ok((response, model_id.clone(), fallback_count));
                }
                Err(e) => {
                    let err_str = e.to_string();
                    println!("    ❌ {} Failed: {}", model_id, err_str);
                    self.health_checker.mark_failure(model_id, &err_str);
                    println!("    🔄 Fallback triggered...");
                    fallback_count += 1;
                }
            }
        }

        Err(anyhow!("All providers for role {} failed.", role))
    }
}
