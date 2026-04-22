use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashSet;

use stylance::import_crate_style;

import_crate_style!(styles, "./src/pages/styles/single.module.scss");

use crate::bangumi::*;
use crate::components::card::Card;
use crate::components::jmp_btn::JmpBtn;
use crate::config::{Config, Language};

#[component]
pub fn Single() -> impl IntoView {
    let config = use_context::<ReadSignal<Config>>().expect("reader");

    let (user_input, set_user_input) = signal("".to_string());
    let (debounced_input, set_debounced_input) = signal("".to_string());
    let (input_version, set_input_version) = signal(0);

    let (input_focused, set_input_focused) = signal(false);
    let (selected_dropdown_index, set_selected_dropdown_index) = signal(0usize);

    let (cards, set_cards) = signal::<Vec<BangumiSubject>>(vec![]);

    // debounce
    Effect::new(move |_| {
        let current_text = user_input.get();
        set_input_version.update(|v| *v += 1);
        let current_version = input_version.get_untracked();

        spawn_local(async move {
            TimeoutFuture::new(500).await;
            if input_version.get_untracked() == current_version {
                set_debounced_input.set(current_text);
                set_selected_dropdown_index.set(0);
            }
        });
    });

    let search_results = LocalResource::new(move || bangumi_search(debounced_input.get()));
    let answer = LocalResource::new(move || fetch_random_anime());

    provide_context(answer);

    let texts = move || match config.get().lang {
        Language::Chinese => (
            "返回",
            "输入你的答案",
            "发送",
        ),
        Language::English => (
            "Back",
            "Input your answer",
            "Send",
        ),
    };

    let unique_search_results = move || {
        let res = search_results.get().flatten().unwrap_or_default();
        let mut seen = HashSet::new();
        res.into_iter()
            .filter(|item| !item.name_cn.is_empty())
            .filter(|item| seen.insert(item.name_cn.clone()))
            .collect::<Vec<_>>()
    };

    let add_selected_or_first = move || {
        let items = unique_search_results();
        if items.is_empty() {
            return;
        }

        let current_idx = selected_dropdown_index.get_untracked();
        let target = items.get(current_idx).or(items.first()).cloned();

        if let Some(subject) = target {
            set_cards.update(|c| c.push(subject));
            set_user_input.set("".to_string());
            set_selected_dropdown_index.set(0);
        }
    };

    let on_keydown = move |ev: leptos::web_sys::KeyboardEvent| {
        let items = unique_search_results();
        if items.is_empty() {
            return;
        }

        let max_idx = items.len().saturating_sub(1);
        let current = selected_dropdown_index.get_untracked();

        match ev.key().as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                let next = if current >= max_idx { 0 } else { current + 1 };
                set_selected_dropdown_index.set(next);
                set_user_input.set(items[next].name_cn.clone());
            }
            "ArrowUp" => {
                ev.prevent_default();
                let prev = if current == 0 { max_idx } else { current - 1 };
                set_selected_dropdown_index.set(prev);
                set_user_input.set(items[prev].name_cn.clone());
            }
            "Enter" => {
                ev.prevent_default();
                add_selected_or_first();
            }
            _ => {}
        }
    };

    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <h1>"Uh oh! Something went wrong!"</h1>
                <ul>
                    {move || errors.get().into_iter().map(|(_, e)| view! { <li>{e.to_string()}</li> }).collect_view()}
                </ul>
            }
        }>
            <main>
                <div class=styles::top_section>
                    <JmpBtn text={move || texts().0} url="/".to_string()/>
                </div>

                <div class=styles::interact_section>
                    <div class=styles::search_wrapper>
                        <div class=styles::input_section>
                            <span> {move || texts().1} </span>
                            <input
                                type="text"
                                bind:value=(user_input, set_user_input)
                                on:focus=move |_| set_input_focused.set(true)
                                on:blur=move |_| set_input_focused.set(false)
                                on:keydown=on_keydown
                            />
                        </div>

                        // the float list
                        {move || {
                            let items = unique_search_results();
                            let focused = input_focused.get();
                            let input_val = user_input.get();

                            if focused && !items.is_empty() && !input_val.is_empty() {
                                view! {
                                    <div>
                                        <ul class=styles::dropdown_list>
                                            <For
                                                each=move || items.clone().into_iter().enumerate()
                                                key=|(_, item)| item.id.clone()
                                                children=move |(i, item)| {
                                                    let is_selected = move || selected_dropdown_index.get() == i;
                                                    let name_clone = item.name_cn.clone();
                                                    view! {
                                                        <li
                                                            class=move || if is_selected() { styles::dropdown_item_active } else { styles::dropdown_item }
                                                            on:mousedown=move |ev| ev.prevent_default()
                                                            on:click=move |_| {
                                                                set_user_input.set(name_clone.clone());
                                                                set_selected_dropdown_index.set(i);
                                                                add_selected_or_first();
                                                            }
                                                        >
                                                            {item.name_cn}
                                                        </li>
                                                    }
                                                }
                                            />
                                        </ul>
                                    </div>
                                }
                            } else {
                                view! { <div> <span style="display:none;"></span> </div>}
                            }
                        }}
                    </div>

                    <div class=styles::button_section>
                        <button on:click=move |_| add_selected_or_first()> {move || texts().2} </button>
                    </div>
                </div>

                <Suspense fallback=move || view! {<p>"Loading answer..."</p>}>
                    {move || Suspend::new(async move {
                        match answer.await {
                            Some(a) => view! { <div> <Card info=a.clone() answer=a/> </div> }.into_view(),
                            None => view! { <div> <p>"nothing"</p> </div> }.into_view()
                        }
                    })}
                </Suspense>

                // all the answers
                <div class=styles::display_section>
                    <Suspense fallback=move || view! {<p>"Loading Results..."</p>}>
                        {move || Suspend::new(async move {
                            let ans_opt = answer.await;
                            match ans_opt {
                                Some(ans) => view! {
                                    <div>
                                        <For
                                            each=move || cards.get()
                                            key=|item| item.id.clone()
                                            children={
                                                let ans_for_closure = ans.clone();
                                                move |item| {
                                                    view! {
                                                        <Card info=item answer=ans_for_closure.clone()/>
                                                    }
                                                }
                                            }
                                        />
                                    </div>
                                }.into_view(),
                                None => view! { <div>"Something wrong here! OMG!!!"</div> }.into_view()
                            }
                        })}
                    </Suspense>
                </div>

                <div class=styles::bottom_section></div>
            </main>
        </ErrorBoundary>
    }
}
