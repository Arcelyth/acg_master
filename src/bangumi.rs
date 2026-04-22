use rand::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BangumiSearchResponse {
    pub data: Vec<BangumiSubject>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BangumiTags {
    pub name: String,
    pub count: usize,
    pub total_cont: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BangumiSubject {
    pub id: usize,
    #[serde(default, deserialize_with = "deserialize_null_to_empty")]
    pub date: String,
    pub image: String,
    pub summary: String,
    pub name: String,
    pub name_cn: String,
    pub images: SubjectImages,
    pub tags: Vec<BangumiTags>,
    pub eps: usize,
    pub total_episodes: usize,
    pub meta_tags: Vec<String>,
    #[serde(rename = "type")]
    pub kind: usize,
}

fn deserialize_null_to_empty<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubjectImages {
    pub common: String,
}

pub struct CompareResult {
    pub correct: HashSet<String>,
    pub close: HashSet<String>,
    pub wrong: HashSet<String>,
}

pub async fn bangumi_search(keyword: String) -> Option<Vec<BangumiSubject>> {
    if keyword.is_empty() {
        return None;
    }

    let client = Client::new();

    let body = serde_json::json!({
        "keyword": keyword,
        "filter": {
            "type": [2],
            "meta_tags": [
                "TV",
                "日本"
            ]
        }
    });

    let url = "https://api.bgm.tv/v0/search/subjects";
    let res = client
        .post(url)
        .header("User-Agent", "LeptosApp/0.1.0")
        .json(&body)
        .send()
        .await
        .ok()?;

    let result: BangumiSearchResponse = res.json().await.ok()?;
    Some(result.data)
}

pub async fn fetch_random_anime() -> Option<BangumiSubject> {
    let client = reqwest::Client::new();
    let mut rng = rand::rng();

    let letters: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
    let random_keyword = letters.choose(&mut rng).unwrap_or(&'a').to_string();

    let random_offset = rng.random_range(0..1000);

    let url = "https://api.bgm.tv/v0/search/subjects";

    let body = serde_json::json!({
         "keyword": random_keyword,
         "filter": {
             "type": [2],
             "meta_tags": [
                 "TV",
                 "日本"
             ]
         },
         "limit": 1,
         "offset": random_offset
    });

    let res = client
        .post(url)
        .header("User-Agent", "LeptosApp/1.0")
        .json(&body)
        .send()
        .await
        .ok()?;
    if res.status() == 200 {
        let result: serde_json::Value = res.json().await.ok()?;
        if let Some(first_item) = result.get("data")?.get(0) {
            return serde_json::from_value(first_item.clone()).ok();
        }
    }
    None
}

pub fn compare_anime(guess: &BangumiSubject, answer: &BangumiSubject) -> CompareResult {
    let mut correct = HashSet::new();
    let mut close = HashSet::new();
    let mut wrong = HashSet::new();

    if guess.name == answer.name {
        correct.insert("name".to_string());
    } else {
        wrong.insert("name".to_string());
    }
    if guess.name_cn == answer.name_cn {
        correct.insert("name_cn".to_string());
    } else {
        wrong.insert("name_cn".to_string());
    }

    if guess.date == answer.date {
        correct.insert("date".to_string());
    } else if !guess.date.is_empty()
        && !answer.date.is_empty()
        && &guess.date[0..4] == &answer.date[0..4]
    {
        close.insert("date".to_string());
    } else {
        wrong.insert("date".to_string());
    }

    if guess.total_episodes == answer.total_episodes {
        correct.insert("total_episodes".to_string());
    } else if (guess.total_episodes as i32 - answer.total_episodes as i32).abs() <= 3 {
        close.insert("total_episodes".to_string());
    } else {
        wrong.insert("total_episodes".to_string());
    }

    CompareResult {
        correct,
        close,
        wrong,
    }
}

pub fn is_guess_right(guess: &BangumiSubject, answer: &BangumiSubject) -> bool {
    if guess.name == answer.name || guess.name_cn == answer.name_cn {
        true
    } else {
        false
    }
}
