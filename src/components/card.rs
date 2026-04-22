use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/card.module.scss");

use crate::bangumi::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Correct,
    Close,
    Wrong,
}

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
                    <span class=get_status_class("name_cn")>{info.name_cn}</span>
                </div>

                <div class=styles::info_row>
                    <span class=get_status_class("date")>{info.date}</span>
                    <span class=get_status_class("total_episodes")>{info.total_episodes} "话"</span>
                </div>

                <div class=styles::tag_row>
                    {info.meta_tags.into_iter().map(|tag| {
                        view! { <span class=styles::meta_badge>{tag}</span> }
                    }).collect_view()}
                </div>

                <div class=styles::tag_row>
                    {info.tags.into_iter().take(8).map(|tag| {
                        view! { <span class=styles::tag_item>{tag.name}</span> }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
