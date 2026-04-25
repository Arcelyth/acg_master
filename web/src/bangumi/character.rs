use std::collections::HashSet;

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BangumiSearchResponse2 {
    pub data: Vec<BangumiCharacter>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BangumiCharacter {
    pub id: usize,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: usize,
    pub summary: String,
    
    pub images: CharacterImages,
    
    pub birth_year: Option<usize>,
    pub birth_mon: Option<usize>,
    pub birth_day: Option<usize>,
    
    #[serde(deserialize_with = "deserialize_null_to_empty", default)]
    pub gender: String,
    #[serde(deserialize_with = "deserialize_null_to_empty", default)]
    pub blood_type: String,
    
    pub infobox: Option<Vec<Info>>,
    pub nsfw: bool,
    pub locked: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CharacterImages {
    pub large: String,
    pub medium: String,
    pub small: String,
    pub grid: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Info {
    pub key: String,
    pub value: serde_json::Value,
}

fn deserialize_null_to_empty<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}


#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CompareResult {
    pub correct: HashSet<String>,
    pub almost: HashSet<String>,
    pub close: HashSet<String>,
    pub wrong: HashSet<String>,
    pub answer_meta_set: HashSet<String>,
    pub answer_tags_set: HashSet<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct GuessResponse {
    pub is_correct: bool,
    pub comparison: CompareResult,
    pub answer: Option<(BangumiCharacter, CompareResult)>,
}

pub async fn bangumi_search_character(keyword: String) -> Option<Vec<BangumiCharacter>> {
    if keyword.is_empty() {
        return None;
    }

    let client = Client::new();

    let body = serde_json::json!({
        "keyword": keyword,
    });

    let url = "https://api.bgm.tv/v0/search/characters";
    let res = client
        .post(url)
        .header(
            "User-Agent",
            "arcelyth/acg_master (https://github.com/arcelyth/acg_master)",
        )
        .json(&body)
        .send()
        .await
        .ok()?;

    let result: BangumiSearchResponse2 = res.json().await.ok()?;
    Some(result.data)
}

