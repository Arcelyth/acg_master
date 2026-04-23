use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/card.module.scss");

use crate::bangumi::*;

#[component]
pub fn Card(
    info: BangumiSubject, 
    answer: BangumiSubject
) -> impl IntoView {
    let comparison = compare_anime(&info, &answer);

    let get_status_class = move |field: &str| {
        if comparison.correct.contains(field) {
            styles::status_correct
        } else if comparison.close.contains(field) {
            styles::status_close
        } else if comparison.almost.contains(field) {
            styles::status_almost
        } else {
            styles::status_wrong
        }
    };

    view! {
        <div class=styles::card_container>
            <div class=styles::card_image>
                <img src=info.images.common alt="cover" />
            </div>

            <div class=styles::card_content>
                <div class=styles::info_row>
                    <span class=get_status_class("name")>{info.name}</span>
                    <span class=get_status_class("name_cn")>{if info.name_cn == "" {"-".to_string()} else {info.name_cn}}</span>
                </div>

                <div class=styles::info_row>
                    <span class=get_status_class("date")>{if info.date == "" {"-".to_string()} else {info.date}}</span>
                    <span class=get_status_class("total_episodes")>{info.total_episodes} "话"</span>
                </div>

                <div class=styles::tag_row>
                    {info.meta_tags.into_iter().map(|tag| {
                        let is_correct = comparison.answer_meta_set.contains(&tag);
                        let meta_style = if is_correct {styles::meta_status_correct} else {styles::meta_status};
                        view! { <span class=meta_style>{tag}</span> }
                    }).collect_view()}
                </div>

                <div class=styles::tag_row>
                    {info.tags.into_iter().map(|tag| {
                        let is_correct = comparison.answer_tags_set.contains(&tag.name);
                        let tag_style = if is_correct {styles::tag_status_correct} else {styles::tag_status};

                        view! { <span class=tag_style>{tag.name}</span> }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
