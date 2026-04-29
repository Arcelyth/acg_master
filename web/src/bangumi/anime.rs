use std::collections::HashSet;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use web_sys::window;

use crate::config::Config;

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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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
        .header(
            "User-Agent",
            "arcelyth/acg_master (https://github.com/arcelyth/acg_master)",
        )
        .json(&body)
        .send()
        .await
        .ok()?;

    let result: BangumiSearchResponse = res.json().await.ok()?;
    Some(result.data)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct StartGameRequest {
    max_guess: usize,
    start_year: usize,
    end_year: usize,
}

pub async fn anime_start_game(config: Config) -> bool {
    let client = Client::new();
    let server_config = StartGameRequest {
        max_guess: config.max_guess,
        start_year: config.start_year,
        end_year: config.end_year,
    };
    let url = if cfg!(debug_assertions) {
        format!("http://localhost:8060/api/bangumi/anime/start_game")
    } else {
        let origin = window().unwrap().location().origin().unwrap();
        format!("{}/api/bangumi/anime/start_game", origin)
    };
    let res = client
        .post(url)
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

    let url = if cfg!(debug_assertions) {
        format!("http://localhost:8060/api/bangumi/anime/verify_guess")
    } else {
        let origin = window().unwrap().location().origin().unwrap();
        format!("{}/api/bangumi/anime/start_game", origin)
    };

    let res = client
        .post(url)
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Diff {
    Right,
    Wrong,
    Close,
    Almost,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BangumiSubjectHide {
    pub date: Diff,
    pub name: bool,
    pub name_cn: bool,
    pub tags: Vec<bool>,
    pub eps: Diff,
    pub total_episodes: Diff,
    pub meta_tags: Vec<bool>,
}

pub fn get_hide_subject(answer: &BangumiSubject, guess: &BangumiSubject) -> BangumiSubjectHide {
    let date_diff = if guess.date == answer.date {
        Diff::Right
    } else if !guess.date.is_empty() && !answer.date.is_empty() {
        let g_year = guess.date.get(0..4).and_then(|s| s.parse::<i32>().ok());
        let a_year = answer.date.get(0..4).and_then(|s| s.parse::<i32>().ok());
        if let (Some(gy), Some(ay)) = (g_year, a_year) {
            let diff = (gy - ay).abs();
            if diff == 0 {
                Diff::Almost
            } else if diff <= 3 {
                Diff::Close
            } else {
                Diff::Wrong
            }
        } else {
            Diff::Wrong
        }
    } else {
        Diff::Wrong
    };

    let calc_eps_diff = |g: usize, a: usize| {
        let diff = (g as i32 - a as i32).abs();
        if diff == 0 {
            Diff::Right
        } else if diff <= 2 {
            Diff::Almost
        } else if diff <= 10 {
            Diff::Close
        } else {
            Diff::Wrong
        }
    };

    let answer_tags_set: HashSet<String> = answer.tags.iter().map(|t| t.name.clone()).collect();
    let tags_res = guess
        .tags
        .iter()
        .map(|t| answer_tags_set.contains(&t.name))
        .collect();

    let answer_meta_set: HashSet<String> = answer.meta_tags.iter().cloned().collect();
    let meta_res = guess
        .meta_tags
        .iter()
        .map(|t| answer_meta_set.contains(t))
        .collect();

    BangumiSubjectHide {
        date: date_diff,
        name: guess.name == answer.name,
        name_cn: guess.name_cn == answer.name_cn,
        tags: tags_res,
        eps: calc_eps_diff(guess.eps, answer.eps),
        total_episodes: calc_eps_diff(guess.total_episodes, answer.total_episodes),
        meta_tags: meta_res,
    }
}
