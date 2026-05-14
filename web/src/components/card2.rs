use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/card2.module.scss");

use crate::bangumi::anime::*;
use crate::config::{Config, Language};

#[component]
pub fn Card2(info: BangumiSubject, comparison: CompareResult) -> impl IntoView {
    let config = use_context::<ReadSignal<Config>>().expect("to have found the reader provided");

    let comp_for_class = comparison.clone();
    let get_status_class = move |field: &str| {
        let up = format!("{}_up", field);
        let down = format!("{}_down", field);
        if comp_for_class.correct.contains(field) {
            styles::status_correct
        } else if comp_for_class.close.contains(field) || comp_for_class.close.contains(&up) || comp_for_class.close.contains(&down) {
            styles::status_close
        } else if comp_for_class.almost.contains(field) || comp_for_class.almost.contains(&up) || comp_for_class.almost.contains(&down) {
            styles::status_almost
        } else {
            styles::status_wrong
        }
    };

    let comp_for_tip = comparison.clone();
    let date_tip = move || {
        if comp_for_tip.close.contains("date") || comp_for_tip.close.contains("date_up") || comp_for_tip.close.contains("date_down") {
            match config.get().lang {
                Language::Chinese => "相差≤3年",
                _ => "Diff ≤3 years",
            }
        } else if comp_for_tip.almost.contains("date") || comp_for_tip.almost.contains("date_up") || comp_for_tip.almost.contains("date_down") {
            match config.get().lang {
                Language::Chinese => "同年",
                _ => "Same year",
            }
        } else {
            ""
        }
    };

    let comp_for_tip2 = comparison.clone();
    let ep_tip = move || {
        if comp_for_tip2.close.contains("total_episodes") || comp_for_tip2.close.contains("total_episodes_up") || comp_for_tip2.close.contains("total_episodes_down") {
            match config.get().lang {
                Language::Chinese => "相差≤10话",
                _ => "Diff ≤10 eps",
            }
        } else if comp_for_tip2.almost.contains("total_episodes") || comp_for_tip2.almost.contains("total_episodes_up") || comp_for_tip2.almost.contains("total_episodes_down") {
            match config.get().lang {
                Language::Chinese => "相差≤2话",
                _ => "Diff ≤2 eps",
            }
        } else {
            ""
        }
    };

    let comp_for_tip3 = comparison.clone();
    let rating_tip = move || {
        if comp_for_tip3.close.contains("rating") || comp_for_tip3.close.contains("rating_up") || comp_for_tip3.close.contains("rating_down") {
            match config.get().lang {
                Language::Chinese => "相差≤2",
                _ => "Diff ≤ 2",
            }
        } else if comp_for_tip3.almost.contains("rating") || comp_for_tip3.almost.contains("rating_up") || comp_for_tip3.almost.contains("rating_down") {
            match config.get().lang {
                Language::Chinese => "相差≤1",
                _ => "Diff ≤ 1",
            }
        } else {
            ""
        }
    };

    let comp4 = comparison.clone();
    let get_arrow_icon = move |field: &str| {
        let up = format!("{}_up", field);
        let down = format!("{}_down", field);
        
        let (is_up, is_down) = if comp4.almost.contains(&up) || comp4.close.contains(&up) {
            (true, false)
        } else if comp4.almost.contains(&down) || comp4.close.contains(&down) {
            (false, true)
        } else {
            (false, false)
        };

        if is_up {
            view! {
                <div>
                <svg class=styles::arrow_icon viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M18 15l-6-6-6 6" />
                </svg>
                </div>
            }
        } else if is_down {
            view! {
                <div>
                <svg class=styles::arrow_icon viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M6 9l6 6 6-6" />
                </svg>
                </div>
            }
        } else {
            view! {<div> <span class=styles::arrow_empty></span> </div>}
        }
    };

    view! {
        <div class=styles::card_container>
            <div class=styles::card_image>
                <img src=info.images.common alt="cover" />
            </div>

            <div class=styles::card_content>
                <div class=styles::col_names>
                    <span class=get_status_class("name")>{info.name.clone()}</span>
                    <span class=get_status_class("name_cn")>{if info.name_cn == "" {"-".to_string()} else {info.name_cn.clone()}}</span>
                    <div class=styles::tooltip_wrapper>
                        <span class=get_status_class("total_episodes")>
                            {info.total_episodes} "话"
                            {get_arrow_icon("total_episodes")}
                        </span>
                        { if ep_tip() != "" {
                            view! {<div class=styles::tip_container><span class=styles::tips> {ep_tip} </span></div>}
                        } else {
                            view! {<div class=styles::tip_container><span> </span></div>}
                        } }
                    </div>

                </div>

                <div class=styles::col_rating>
                   <div class=styles::tooltip_wrapper>
                        <span class=get_status_class("rating")>
                            {if info.rating.score == 0. {"-".to_string()} else {info.rating.score.to_string()} }
                            {get_arrow_icon("rating")}
                        </span>
                        { if rating_tip() != "" {
                            view! {<div class=styles::tip_container><span class=styles::tips> {rating_tip} </span></div>}
                        } else {
                            view! {<div class=styles::tip_container><span> </span></div>}
                        } }
                    </div>
                </div>

                <div class=styles::col_date>
                    <div class=styles::tooltip_wrapper>
                        <span class=get_status_class("date")>
                            {if info.date == "" {"-".to_string()} else {info.date.clone()}}
                            {get_arrow_icon("date")}
                        </span>
                        { if date_tip() != "" {
                            view! {<div class=styles::tip_container><span class=styles::tips> {date_tip} </span></div>}
                        } else {
                            view! {<div class=styles::tip_container><span> </span></div>}
                        } }
                    </div>
                </div>

    
                <div class=styles::col_tags>
                    {info.tags.into_iter().map({
                        let comp_for_tags = comparison.clone();
                        move |tag| {
                            let is_correct = comp_for_tags.answer_tags_set.contains(&tag.name);
                            let tag_style = if is_correct {styles::tag_status_correct} else {styles::tag_status};

                            view! { <span class=tag_style>{tag.name}</span> }
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
