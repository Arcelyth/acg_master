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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GameState {
    Loading,
    Playing,
    Win,
    Lose,
}

#[component]
pub fn Single() -> impl IntoView {
    let config = use_context::<ReadSignal<Config>>().expect("reader");

    let (user_input, set_user_input) = signal("".to_string());
    let (debounced_input, set_debounced_input) = signal("".to_string());
    let (input_version, set_input_version) = signal(0);

    let (input_focused, set_input_focused) = signal(false);
    let (selected_dropdown_index, set_selected_dropdown_index) = signal(0usize);

    let (guess_time, set_guess_time) = signal(0usize);
    let (game_state, set_game_state) = signal(GameState::Loading);
    let (dup, set_dup) = signal(false);

    let (cards, set_cards) = signal::<Vec<BangumiSubject>>(vec![]);
    let search_results = LocalResource::new(move || bangumi_search(debounced_input.get()));
    let (refresh_trigger, set_refresh_trigger) = signal(0);
    let answer = LocalResource::new(move || {
        refresh_trigger.get();
        fetch_random_anime()
    });


    // Loading -> Playing
    Effect::new(move |_| {
        if let Some(Some(_)) = answer.get() {
            if game_state.get_untracked() == GameState::Loading {
                set_game_state.set(GameState::Playing);
            }
        }
    });

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

    let texts = move || match config.get().lang {
        Language::Chinese => ("返回", "输入你的答案", "发送"),
        Language::English => ("Back", "Input your answer", "Send"),
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
        set_dup.set(false);
        let items = unique_search_results();
        if items.is_empty() {
            return;
        }

        let current_idx = selected_dropdown_index.get_untracked();
        let target = items.get(current_idx).or(items.first()).cloned();

        if let Some(subject) = target {
            let exists = cards.get_untracked().iter().any(|c| c.id == subject.id);
            if exists {
                set_dup.set(true);
                return;
            }
            set_cards.update(|c| c.push(subject.clone()));
            set_user_input.set("".to_string());
            set_selected_dropdown_index.set(0);
            let ans_len = cards.get().len();
            set_guess_time.set(ans_len);
            if let Some(Some(actual_answer)) = answer.get_untracked() {
                if is_guess_right(&subject, &actual_answer) {
                    set_game_state.set(GameState::Win);
                    return;
                }
            }

            if ans_len >= config.get().max_guess {
                set_game_state.set(GameState::Lose);
            }
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
            }
            "ArrowUp" => {
                ev.prevent_default();
                let prev = if current == 0 { max_idx } else { current - 1 };
                set_selected_dropdown_index.set(prev);
            }
            "Enter" => {
                ev.prevent_default();
                if let Some(item) = items.get(current) {
                    set_user_input.set(item.name_cn.clone());
                }
            }
            _ => {}
        }
    };

    let is_interaction_disabled = move || game_state.get() != GameState::Playing;

    let reset_game = move |_| {
        set_cards.set(vec![]);
        set_guess_time.set(0);
        set_user_input.set("".to_string());
        set_dup.set(false);

        set_game_state.set(GameState::Loading);

        set_refresh_trigger.update(|n| *n += 1);
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
                    // return button
                    <JmpBtn text={move || texts().0} url="/".to_string()/>
                </div>

                <div class=styles::interact_section>
                    <div class=styles::search_wrapper>
                        <div class=styles::input_section>
                            <span> {move || texts().1}: </span>
                            <input
                                type="text"
                                disabled=is_interaction_disabled
                                bind:value=(user_input, set_user_input)
                                on:focus=move |_| set_input_focused.set(true)
                                on:blur=move |_| set_input_focused.set(false)
                                on:keydown=on_keydown
                            />
                        </div>
                        {move || {
                            if dup.get() {
                                view! { <div><span class=styles::dup_message>"已在列表中"</span></div> }
                            } else {
                                view! { <div><div style="display:none"></div></div> }
                            }
                        }}

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
                        // send buttons
                        <button
                            disabled=is_interaction_disabled
                            on:click=move |_| add_selected_or_first()
                        >
                            {move || texts().2}
                        </button>
                        // reset button
                        <button
                            class=styles::reset_btn
                            on:click=reset_game
                        > </button>
                        </div>
                    <div class=styles::guess_number>
                        <span> {guess_time}/{config.get().max_guess} </span>
                    </div>
                </div>

                <Suspense fallback=move || view! {<p>"Loading..."</p>}>
                    {move || Suspend::new(async move {
                        match answer.await {
                            Some(a) => view! { <div> <Card info=a.clone() answer=a/> </div> }.into_view(),
                            None => view! { <div> <p>"nothing"</p> </div> }.into_view()
                        }
                    })}
                </Suspense>

                // all the answers
                <div class=styles::display_section>
                    <Suspense fallback=move || view! {<p>"Loading..."</p>}>
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

                <div class=styles::answer_reveal_section>
                    {move || {
                        let state = game_state.get();
                        if state == GameState::Win || state == GameState::Lose {
                            let (status_text, status_class) = match state {
                                GameState::Win => ("？！强强！？", styles::status_win),
                                GameState::Lose => ("？！弱弱！？", styles::status_lose),
                                _ => ("", ""),
                            };

                            view! {
                                <div>
                                    <div class=styles::reveal_container>
                                        <h2 class=status_class>{status_text}</h2>
                                        <h4 class=status_class>{guess_time}/{config.get().max_guess}</h4>
                                        <button
                                            class=styles::reset_btn
                                            on:click=reset_game
                                        > </button>
                                        <hr class=styles::divider />
                                        <p class=styles::reveal_text> "ANSWER" </p>

                                        <Suspense fallback=|| view! { "..." }>
                                            {move || Suspend::new(async move {
                                                match answer.await {
                                                    Some(a) => view! {<div> <Card info=a.clone() answer=a/> </div>},
                                                    None => view! { <div>"Nothing"</div> }
                                                }
                                            })}
                                        </Suspense>
                                    </div>
                                </div>
                            }
                        } else {
                            view! { <div><div style="display:none"></div></div> }
                        }
                    }}
                </div>

                <div class=styles::bottom_section></div>
            </main>
        </ErrorBoundary>
    }
}
