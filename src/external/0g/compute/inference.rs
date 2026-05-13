// src/external/0g/compute/inference.rs
// 0G Compute inference client — OpenAI-compatible API

use std::time::Duration;

use crate::config::CONFIG;

/// Full gameplay intelligence analysis returned by 0G Compute.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MomentAnalysis {
    /// One punchy sentence about the moment (max 120 chars)
    pub caption: String,

    /// Overall rank score 0-100. 90+ = exceptional, 70-89 = strong, 40-69 = good, <40 = standard
    #[serde(rename = "rankScore")]
    pub rank_score: u32,

    /// Up to 3 specific gaming achievement tags
    pub highlights: Vec<String>,

    /// Gameplay classification: "clutch" | "speedrun" | "strategy" | "ai_duel" | "domination" | "highlight"
    #[serde(rename = "momentType", default)]
    pub moment_type: Option<String>,

    /// Skill assessment 0-100 (reaction speed, precision, decision quality)
    #[serde(rename = "skillScore", default)]
    pub skill_score: Option<u32>,

    /// Reaction quality: "low" | "medium" | "high" | "exceptional"
    #[serde(rename = "reactionQuality", default)]
    pub reaction_quality: Option<String>,

    /// Rarity tier: "common" | "rare" | "epic" | "legendary"
    #[serde(default)]
    pub rarity: Option<String>,
}

pub struct ZgComputeClient {
    client: reqwest::Client,
    provider_url: String,
    api_key: String,
    model: String,
}

impl ZgComputeClient {
    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn from_config() -> Option<Self> {
        let provider_url = CONFIG.zg.compute_provider_url.clone()?;
        let api_key = CONFIG.zg.compute_api_key.clone()?;
        let model = CONFIG.zg.compute_model.clone();
        Some(Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(45))
                .build()
                .unwrap_or_default(),
            provider_url,
            api_key,
            model,
        })
    }

    pub async fn analyze_moment(
        &self,
        title: &str,
        description: Option<&str>,
        tags: &[String],
        related_games: &[String],
    ) -> Result<MomentAnalysis, String> {
        let prompt = format!(
            "You are a gaming moment analyst for the Kult Web3 platform, verified on 0G Network. \
            Gaming moments are on-chain clips and screenshots from Kult's games:\n\
            - guess-the-ai: player detects AI-generated content (skill = deduction speed, precision)\n\
            - highway-hustle: speed racing game (skill = reaction time, lap consistency)\n\
            - ai-arena: AI agent battle (skill = strategy, agent build quality)\n\
            - kult-royale: battle royale (skill = survival decisions, kill quality)\n\n\
            Analyze this moment and respond with ONLY valid JSON — no markdown, no explanation.\n\n\
            Moment:\n\
            Title: {}\n\
            Description: {}\n\
            Tags: {}\n\
            Games: {}\n\n\
            rankScore guide:\n\
            90-100 = exceptional (record-breaking, perfect play, viral-worthy)\n\
            70-89  = strong (clutch win, impressive skill, notable achievement)\n\
            40-69  = good (solid play, fun moment, worth sharing)\n\
            0-39   = standard (ordinary clip, participation)\n\n\
            Respond with exactly this JSON:\n\
            {{\
            \"caption\": \"<one punchy present-tense sentence, max 120 chars>\",\
            \"rankScore\": <integer 0-100>,\
            \"highlights\": [<1-3 specific strings e.g. \"perfect run\", \"AI deceived\", \"clutch kill\">],\
            \"momentType\": <\"clutch\"|\"speedrun\"|\"strategy\"|\"ai_duel\"|\"domination\"|\"highlight\">,\
            \"skillScore\": <integer 0-100>,\
            \"reactionQuality\": <\"low\"|\"medium\"|\"high\"|\"exceptional\">,\
            \"rarity\": <\"common\"|\"rare\"|\"epic\"|\"legendary\">\
            }}",
            title,
            description.unwrap_or("No description provided"),
            if tags.is_empty() { "none".to_string() } else { tags.join(", ") },
            if related_games.is_empty() { "none".to_string() } else { related_games.join(", ") },
        );

        let url = format!(
            "{}/v1/proxy/chat/completions",
            self.provider_url.trim_end_matches('/')
        );

        let body = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.3,
            "max_tokens": 320
        });

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("0G Compute request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!(
                "0G Compute error {}: {}",
                status,
                &text[..text.len().min(300)]
            ));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse 0G Compute response: {}", e))?;

        let content = raw
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| "Empty or unexpected 0G Compute response".to_string())?;

        // Strip markdown code fences if the LLM adds them
        let content = content.trim();
        let content = content
            .strip_prefix("```json")
            .or_else(|| content.strip_prefix("```"))
            .unwrap_or(content)
            .strip_suffix("```")
            .unwrap_or(content)
            .trim();

        serde_json::from_str::<MomentAnalysis>(content).map_err(|e| {
            format!(
                "Failed to parse AI JSON: {} — raw: {}",
                e,
                &content[..content.len().min(300)]
            )
        })
    }
}
