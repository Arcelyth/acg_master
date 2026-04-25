use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BangumiSearchResponse {
    pub data: Vec<BangumiSubject>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BangumiTags {
    pub name: String,
    pub count: usize,
    pub total_cont: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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
    #[serde(rename = "type", alias = "kind")]
    pub kind: usize,
}

fn deserialize_null_to_empty<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SubjectImages {
    pub common: String,
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
    pub answer: Option<(BangumiSubject, CompareResult)>,
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
        .header("User-Agent", "arcelyth/acg-master (https://github.com/arcelyth/acg_master)")
        .json(&body)
        .send()
        .await
        .ok()?;

    let result: BangumiSearchResponse = res.json().await.ok()?;
    Some(result.data)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct StartGameRequest {
    max_guess: usize,
}

pub async fn anime_start_game(max_guess: usize) -> bool {
    let client = Client::new();
    let server_config = StartGameRequest { max_guess };
    let res = client
        .post("http://127.0.0.1:8066/api/bangumi/anime/start_game")
        .fetch_credentials_include()
        .json(&server_config)
        .send()
        .await;

    if let Ok(response) = res {
        if response.status() == 200 {
            true
        } else {
            false
        }
    } else {
        false
    }
}

pub async fn compare_anime(guess: &BangumiSubject) -> GuessResponse {
    let client = Client::builder().build().unwrap();

    let res = client
        .post("http://127.0.0.1:8066/api/bangumi/anime/verify_guess")
        .fetch_credentials_include()
        .json(guess)
        .send()
        .await;
    if let Ok(response) = res {
        if response.status() == 200 {
            if let Ok(result) = response.json::<GuessResponse>().await {
                return result;
            }
        }
    }
    GuessResponse {
        is_correct: false,
        comparison: CompareResult {
            correct: HashSet::new(),
            almost: HashSet::new(),
            close: HashSet::new(),
            wrong: HashSet::new(),
            answer_meta_set: HashSet::new(),
            answer_tags_set: HashSet::new(),
        },
        answer: None,
    }
}

pub fn is_guess_right(guess: &BangumiSubject, answer: &BangumiSubject) -> bool {
    if guess.name == answer.name || guess.name_cn == answer.name_cn {
        true
    } else {
        false
    }
}
