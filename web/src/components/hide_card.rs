use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/hide_card.module.scss");

use crate::bangumi::anime::{BangumiSubjectHide, Diff};

#[component]
pub fn HideCard(hide: BangumiSubjectHide) -> impl IntoView {
    let get_bool_class = |is_right: bool| {
        if is_right {
            styles::status_correct
        } else {
            styles::status_wrong
        }
    };

    let get_diff_class = |diff: &Diff| {
        match diff {
            Diff::Right => styles::status_correct,
            Diff::CloseUp | Diff::CloseDown => styles::status_close,
            Diff::AlmostUp | Diff::AlmostDown => styles::status_almost,
            Diff::Wrong => styles::status_wrong,
        }
    };

    view! {
        <div class=styles::hide_card_container>
            <div class=styles::card_image></div>

            <div class=styles::card_content>
                <div class=styles::info_row>
                    <span class=format!("{} {}", get_bool_class(hide.name), styles::ph_name)></span>
                    <span class=format!("{} {}", get_bool_class(hide.name_cn), styles::ph_name_cn)></span>
                </div>

                <div class=styles::info_row>
                    <div class=styles::tooltip_wrapper>
                        <span class=format!("{} {}", get_diff_class(&hide.date), styles::ph_date)></span>
                    </div>

                    <div class=styles::tooltip_wrapper>
                        <span class=format!("{} {}", get_diff_class(&hide.total_episodes), styles::ph_eps)></span>
                    </div>
                </div>

                <div class=styles::tag_row>
                    {hide.meta_tags.into_iter().map(|is_correct| {
                        view! { <span class=format!("{} {}", get_bool_class(is_correct), styles::ph_meta)></span> }
                    }).collect_view()}
                </div>

                <div class=styles::tag_row>
                    {hide.tags.into_iter().map(|is_correct| {
                        view! { <span class=format!("{} {}", get_bool_class(is_correct), styles::ph_tag)></span> }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
