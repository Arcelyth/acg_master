use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/hide_card2.module.scss");

use crate::bangumi::anime::{BangumiSubjectHide, Diff};

#[component]
pub fn HideCard2(hide: BangumiSubjectHide) -> impl IntoView {
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

    let get_arrow_icon = |diff: &Diff| {
        match diff {
            Diff::CloseUp | Diff::AlmostUp => {
                view! {
                    <div>
                        <svg class=styles::arrow_icon viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M18 15l-6-6-6 6" />
                        </svg>
                    </div>
                }.into_any()
            },
            Diff::CloseDown | Diff::AlmostDown => {
                view! {
                    <div>
                        <svg class=styles::arrow_icon viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M6 9l6 6 6-6" />
                        </svg>
                    </div>
                }.into_any()
            },
            _ => view! { <div><span class=styles::arrow_empty></span></div> }.into_any(),
        }
    };

    view! {
        <div class=styles::card_container>
            <div class=styles::card_image>
            </div>

            <div class=styles::card_content>
                <div class=styles::col_names>
                    <span class=format!("{} {}", get_bool_class(hide.name), styles::ph_name)></span>
                    <span class=format!("{} {}", get_bool_class(hide.name_cn), styles::ph_name_cn)></span>
                    <div class=styles::tooltip_wrapper>
                        <span class=format!("{} {}", get_diff_class(&hide.total_episodes), styles::ph_eps)>
                            {get_arrow_icon(&hide.total_episodes)}
                        </span>
                    </div>
                </div>

                <div class=styles::col_rating>
                    <div class=styles::tooltip_wrapper>
                        <span class=format!("{} {}", styles::status_wrong, styles::ph_rating)></span>
                    </div>
                </div>

                <div class=styles::col_date>
                    <div class=styles::tooltip_wrapper>
                        <span class=format!("{} {}", get_diff_class(&hide.date), styles::ph_date)>
                            {get_arrow_icon(&hide.date)}
                        </span>
                    </div>
                </div>

                <div class=styles::col_tags>
                    {hide.meta_tags.into_iter().map(|is_correct| {
                        view! { <span class=format!("{} {}", get_bool_class(is_correct), styles::ph_meta)></span> }
                    }).collect_view()}

                    {hide.tags.into_iter().map(|is_correct| {
                        view! { <span class=format!("{} {}", get_bool_class(is_correct), styles::ph_tag)></span> }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
