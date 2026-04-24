use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/card.module.scss");

use crate::bangumi::*;
use crate::config::{Config, Language};

#[component]
pub fn Card(info: BangumiSubject, answer: BangumiSubject) -> impl IntoView {
    let comparison = compare_anime(&info, &answer);
    let config = use_context::<ReadSignal<Config>>().expect("to have found the reader provided");
    let comp_for_class = comparison.clone();
    let get_status_class = move |field: &str| {
        if comp_for_class.correct.contains(field) {
            styles::status_correct
        } else if comp_for_class.close.contains(field) {
            styles::status_close
        } else if comp_for_class.almost.contains(field) {
            styles::status_almost
        } else {
            styles::status_wrong
        }
    };
    
    let comp_for_tip = comparison.clone();
    let date_tip = move || {
        if comp_for_tip.close.contains("date") {
            match config.get().lang {
                Language::Chinese => "相差≤3年",
                _ => "Diff ≤3 years",
            }
        } else if comp_for_tip.almost.contains("date") {
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
        if comp_for_tip2.close.contains("total_episodes") {
            match config.get().lang {
                Language::Chinese => "相差≤10话",
                _ => "Diff ≤10 eps",
            }
        } else if comp_for_tip2.almost.contains("total_episodes") {
            match config.get().lang {
                Language::Chinese => "相差≤2话",
                _ => "Diff ≤2 eps",
            }
        } else {
            ""
        }
    };

    view! {
        <div class=styles::card_container>
            <div class=styles::card_image>
                <img src=info.images.common alt="cover" />
            </div>

            <div class=styles::card_content>
                <div class=styles::info_row>
                    <span class=get_status_class("name")>{info.name.clone()}</span>
                    <span class=get_status_class("name_cn")>{if info.name_cn == "" {"-".to_string()} else {info.name_cn.clone()}}</span>
                </div>

                <div class=styles::info_row>
                    <div class=styles::tooltip_wrapper>
                        <span class=get_status_class("date")>
                            {if info.date == "" {"-".to_string()} else {info.date.clone()}}
                        </span>
                        { if date_tip() != "" {
                            view! {<div class=styles::tip_container><span class=styles::tips> {date_tip} </span></div>}
                        } else {
                            view! {<div class=styles::tip_container><span> </span></div>}
                        } }
                    </div>

                    <div class=styles::tooltip_wrapper>
                        <span class=get_status_class("total_episodes")>
                            {info.total_episodes} "话"
                        </span>
                        { if ep_tip() != "" {
                            view! {<div class=styles::tip_container><span class=styles::tips> {ep_tip} </span></div>}
                        } else {
                            view! {<div class=styles::tip_container><span> </span></div>}
                        } }
                    </div>
                </div>

                <div class=styles::tag_row>
                    {info.meta_tags.into_iter().map({
                        let comp_for_meta = comparison.clone();
                        move |tag| {
                            let is_correct = comp_for_meta.answer_meta_set.contains(&tag);
                            let meta_style = if is_correct {styles::meta_status_correct} else {styles::meta_status};
                            view! { <span class=meta_style>{tag}</span> }
                        }
                    }).collect_view()}
                </div>

                <div class=styles::tag_row>
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
