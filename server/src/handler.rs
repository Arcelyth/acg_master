use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;


#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BangumiTags {
    pub name: String,
    pub count: usize,
    pub total_cont: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SubjectImages {
    pub common: String,
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
    #[serde(rename = "type")]
    pub kind: usize,
}

impl BangumiSubject {
    pub fn is_sequel(&self) -> bool {
        self.tags
            .iter()
            .any(|tag| tag.name == "续集" || tag.name == "续作")
    }
}

fn deserialize_null_to_empty<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CompareResult {
    pub correct: HashSet<String>,
    pub almost: HashSet<String>,
    pub close: HashSet<String>,
    pub wrong: HashSet<String>,
    pub answer_meta_set: HashSet<String>,
    pub answer_tags_set: HashSet<String>,
}

#[derive(Serialize)]
pub struct GuessResponse {
    pub is_correct: bool,
    pub comparison: CompareResult,
}

pub async fn fetch_random_anime() -> Option<BangumiSubject> {
    let client = reqwest::Client::new();
    let mut rng = rand::rng();
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 15;

    let year_weights = [
        (2024..=2026, 52),
        (2020..=2023, 30),
        (2010..=2019, 13),
        (2000..=2009, 5),
        (1960..=1999, 1),
    ];
    let dist = WeightedIndex::new(year_weights.iter().map(|item| item.1)).ok()?;

    loop {
        attempts += 1;
        if attempts > MAX_ATTEMPTS {
            return None;
        }

        let selected_range = &year_weights[dist.sample(&mut rng)].0;
        let random_year = rng.random_range(selected_range.clone());

        let url = "https://api.bgm.tv/v0/search/subjects";
        let req_body = serde_json::json!({
            "keyword": "",
            "filter": {
                "type": [2],
                "meta_tags": ["TV", "日本"],
                "air_date": [format!(">={}-01-01", random_year), format!("<={}-12-31", random_year)]
            },
        });

        let count_res = client
            .post(format!("{}?limit=1&offset=0", url))
            .header("User-Agent", "LeptosApp/1.0")
            .json(&req_body)
            .send()
            .await
            .ok()?;
        let total: u32 = if count_res.status() == 200 {
            let json: serde_json::Value = count_res.json().await.unwrap_or_default();
            json.get("total").and_then(|t| t.as_u64()).unwrap_or(0) as u32
        } else {
            continue;
        };

        if total == 0 {
            continue;
        }

        let random_offset = rng.random_range(0..total);
        let res = client
            .post(format!("{}?limit=1&offset={}", url, random_offset))
            .header("User-Agent", "LeptosApp/1.0")
            .json(&req_body)
            .send()
            .await
            .ok()?;

        if res.status() != 200 {
            continue;
        }
        let result: serde_json::Value = res.json().await.unwrap_or_default();

        if let Some(first_item) = result.get("data").and_then(|d| d.get(0)) {
            let subject: BangumiSubject = match serde_json::from_value(first_item.clone()) {
                Ok(s) => s,
                Err(_) => continue,
            };

            if subject.name_cn.trim().is_empty()
                || subject.is_sequel()
                || subject.eps == 0
                || subject.total_episodes == 0
            {
                continue;
            }
            return Some(subject);
        }
    }
}

pub fn compare_anime(guess: &BangumiSubject, answer: &BangumiSubject) -> CompareResult {
    let mut correct = HashSet::new();
    let mut almost = HashSet::new();
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
    } else if !guess.date.is_empty() && !answer.date.is_empty() {
        let g_year = guess.date.get(0..4).and_then(|s| s.parse::<i32>().ok());
        let a_year = answer.date.get(0..4).and_then(|s| s.parse::<i32>().ok());
        if let (Some(gy), Some(ay)) = (g_year, a_year) {
            let diff = (gy - ay).abs();
            if diff == 0 {
                almost.insert("date".to_string());
            } else if diff <= 3 {
                close.insert("date".to_string());
            } else {
                wrong.insert("date".to_string());
            }
        } else {
            wrong.insert("date".to_string());
        }
    } else {
        wrong.insert("date".to_string());
    }

    let ep_diff = (guess.total_episodes as i32 - answer.total_episodes as i32).abs();
    if ep_diff == 0 {
        correct.insert("total_episodes".to_string());
    } else if ep_diff <= 2 {
        almost.insert("total_episodes".to_string());
    } else if ep_diff <= 10 {
        close.insert("total_episodes".to_string());
    } else {
        wrong.insert("total_episodes".to_string());
    }

    CompareResult {
        correct,
        almost,
        close,
        wrong,
        answer_meta_set: answer.meta_tags.iter().cloned().collect(),
        answer_tags_set: answer.tags.iter().map(|t| t.name.clone()).collect(),
    }
}

pub fn is_guess_right(guess: &BangumiSubject, answer: &BangumiSubject) -> bool {
    guess.name == answer.name || guess.name_cn == answer.name_cn
}

pub async fn start_new_game(session: Session) -> impl Responder {
    if let Some(subject) = fetch_random_anime().await {
        if session.insert("current_answer", &subject).is_err() {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Session error"}));
        }
        HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
        }))
    } else {
        HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to fetch anime"}))
    }
}

pub async fn verify_guess(session: Session, guess: web::Json<BangumiSubject>) -> impl Responder {
    match session.get::<BangumiSubject>("current_answer") {
        Ok(Some(answer)) => {
            let is_correct = is_guess_right(&guess, &answer);
            let comparison = compare_anime(&guess, &answer);

            if is_correct {
                session.remove("current_answer");
            }

            HttpResponse::Ok().json(GuessResponse {
                is_correct,
                comparison,
            })
        }
        Ok(None) => HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "No active game found. Please start a new game."})),
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Session read error"})),
    }
}
