use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashSet;
use std::time::Duration;

use stylance::import_crate_style;

import_crate_style!(styles, "./src/pages/styles/single.module.scss");

use crate::bangumi::anime::*;
use crate::components::back_btn::BackBtn;
use crate::components::card2::Card2;
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
    let (dup, set_dup) = signal(false);
    let (guess_time, set_guess_time) = signal(0usize);
    let (game_state, set_game_state) = signal(GameState::Loading);

    let (cards, set_cards) = signal::<Vec<(BangumiSubject, CompareResult)>>(vec![]);
    let (_refresh_trigger, set_refresh_trigger) = signal(0);
    let (answer, set_answer) = signal(None);

    let search_results = LocalResource::new(move || bangumi_search(debounced_input.get()));
    // timer
    let (elapsed_seconds, set_elapsed_seconds) = signal(0u64);
    let (is_timer_running, set_is_timer_running) = signal(false);
    let (current_config, set_current_config) = signal(config.get_untracked());
    let (is_modal_open, set_is_modal_open) = signal(false);
    let (guess_count, set_guess_count) = signal(10usize);
    let (start_year, set_start_year) = signal(1960usize);
    let (end_year, set_end_year) = signal(2026usize);
    let set_config = use_context::<WriteSignal<Config>>().expect("setter");

    let formatted_time = move || {
        let s = elapsed_seconds.get();
        format!("{:02}:{:02}", s / 60, s % 60)
    };

    use_interval(1000, move || {
        if is_timer_running.get() {
            set_elapsed_seconds.update(|sec| *sec += 1);
        }
    });

    Effect::new(move |_| {
        let state = game_state.get();
        if state == GameState::Win || state == GameState::Lose {
            set_is_timer_running.set(false);
        }
    });

    // Loading -> Playing
    Effect::new(move |_| {
        if game_state.get() == GameState::Loading {
            spawn_local(async move {
                let success = anime_start_game(current_config.get_untracked()).await;
                if success {
                    set_game_state.set(GameState::Playing);
                }
            });
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
        Language::Chinese => (
            "返回",
            "输入你的答案",
            "发送",
            "？！强强！？",
            "？！弱弱！？",
            "答案",
            "输入动漫名称",
            "已在列表中",
            ("封面", "标题与集数", "评分", "放送日期", "标签"),
            ("猜测次数", "重置以生效", "年份范围", "至"),
        ),
        Language::English => (
            "Back",
            "Input your answer",
            "Send",
            "?!Strong Strong!?",
            "?!Weak Weak!?",
            "ANSWER",
            "Input anime's name",
            "Already in the list",
            ("Cover", "Title & Eps", "Rating", "Air Date", "Tags"),
            ("Guess Times", "Reset to apply", "Year Range", "to"),
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
        set_dup.set(false);
        let items = unique_search_results();
        if items.is_empty() {
            return;
        }

        let current_idx = selected_dropdown_index.get_untracked();
        let target = items.get(current_idx).or(items.first()).cloned();

        if let Some(subject) = target {
            let exists = cards
                .get_untracked()
                .iter()
                .any(|(c, _)| c.id == subject.id);
            if exists {
                set_dup.set(true);
                return;
            }

            if cards.get_untracked().is_empty() {
                set_is_timer_running.set(true);
            }

            set_user_input.set("".to_string());
            set_selected_dropdown_index.set(0);

            spawn_local(async move {
                let comp_result = compare_anime(&subject).await;

                let is_win = comp_result.is_correct;

                set_cards.update(|c| c.push((subject.clone(), comp_result.comparison)));
                if let Some(ans) = comp_result.answer {
                    set_answer.set(Some(ans));
                }
                let ans_len = cards.get_untracked().len();
                set_guess_time.set(ans_len);

                if is_win {
                    set_game_state.set(GameState::Win);
                } else if ans_len >= current_config.get_untracked().max_guess {
                    set_game_state.set(GameState::Lose);
                }
            });
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

        set_current_config.set(config.get_untracked());

        set_is_timer_running.set(false);
        set_elapsed_seconds.set(0);

        set_refresh_trigger.update(|n| *n += 1);
    };

    let reset_icon = move || {
        view! {
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
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
                    // return button
                    <BackBtn />
                </div>

                <div class=styles::interact_section>
                    <div class=styles::search_wrapper>
                       <div class=styles::input_section>
                            <span> {move || texts().1}: </span>

                            <div class=styles::input_container>
                                <input
                                    placeholder={move || texts().6}
                                    type="text"
                                    disabled=is_interaction_disabled
                                    bind:value=(user_input, set_user_input)
                                    on:focus=move |_| set_input_focused.set(true)
                                    on:blur=move |_| set_input_focused.set(false)
                                    on:keydown=on_keydown
                                />
                                {move || {
                                    if dup.get() {
                                        view! { <div><span class=styles::dup_message>{move || texts().7}</span></div> }
                                    } else {
                                        view! { <div><div style="display:none"></div></div> }
                                    }
                                }}
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
                                        view! { <div><ul style="display:none"></ul></div> }
                                    }
                                }}
                            </div>
                        </div>
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
                        >
                            {reset_icon}
                        </button>

                        </div>
                    <div class=styles::guess_number>
                        <span> { move || format!("{}/{}", guess_time.get(), current_config.get().max_guess) }</span>
                    </div>
                    <div class=styles::timer>
                        <span class=styles::timer_text> {formatted_time} </span>
                    </div>
                </div>
                        // setting button
                        <button
                            class=styles::settings_btn
                            on:click=move |_| set_is_modal_open.set(true)
                        >
                            <svg viewBox="0 0 24 24">
                                <circle cx="12" cy="12" r="3"></circle>
                                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
                            </svg>
                        </button>

                        <Show when=move || is_modal_open.get()>
                            <div class=styles::modal_overlay on:click=move |_| set_is_modal_open.set(false)>
                                <div class=styles::modal_content on:click=move |e| e.stop_propagation()>
                                    <button class=styles::close_btn on:click=move |_|{
                                        set_is_modal_open.set(false);
                                        set_config.update(|v| {
                                            v.max_guess = guess_count.get();
                                            v.start_year = start_year.get();
                                            v.end_year = end_year.get();
                                        });
                                    }>
                                        <svg viewBox="0 0 24 24">
                                            <line x1="18" y1="6" x2="6" y2="18"></line>
                                            <line x1="6" y1="6" x2="18" y2="18"></line>
                                        </svg>
                                    </button>

                                    <div class=styles::setting_item>
                                        <label>{move || texts().9.0}</label>
                                        <div class=styles::slider_wrapper>
                                            <input
                                                type="range"
                                                min="1"
                                                max="100"
                                                prop:value=move || guess_count.get()
                                                on:input=move |ev| {
                                                    if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                                                        set_guess_count.set(val);
                                                    }
                                                }
                                            />
                                            <span>{move || guess_count.get()}</span>
                                        </div>
                                    </div>

                                    <div class=styles::setting_item>
                                        <label>{move || texts().9.2}</label>
                                        <div class=styles::year_range_wrapper>
                                            <input
                                                type="number"
                                                class=styles::year_input
                                                min="1960"
                                                max=move || end_year.get().to_string()
                                                prop:value=move || start_year.get()
                                                on:input=move |ev| {
                                                    if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                                                        set_start_year.set(val);
                                                    }
                                                }
                                            />
                                            <span>{move || texts().9.3}</span>
                                            <input
                                                type="number"
                                                class=styles::year_input
                                                min=move || start_year.get().to_string()
                                                max="2026"
                                                prop:value=move || end_year.get()
                                                on:input=move |ev| {
                                                    if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                                                        set_end_year.set(val);
                                                    }
                                                }
                                            />
                                        </div>
                                    </div>

                                    <div class=styles::note_text>{move || texts().9.1}</div>
                                </div>
                            </div>
                        </Show>

                // all the answers
                <div class=styles::display_section>
                // the table header
                <div class=styles::table_header>
                    <div class=styles::header_image_placeholder>
                        {move || texts().8.0}
                    </div>

                    <div class=styles::header_content_grid>
                        <div class=styles::col_header_text>{move || texts().8.1}</div>
                        <div class=styles::center_text>{move || texts().8.2}</div>
                        <div class=styles::center_text>{move || texts().8.3}</div>
                        <div class=styles::col_header_text>{move || texts().8.4}</div>
                    </div>
                </div>
                <For
                    each=move || cards.get()
                    key=|(item, _)| item.id.clone()
                    children=move |(item, comp_res)| {
                        view! {
                                <Card2 info=item comparison=comp_res />
                            }
                        }
                />
                </div>

                <Show
                    when=move || game_state.get() == GameState::Loading
                    fallback=move || view! { <div /> }
                >
                    <div class=styles::loader_wrapper>
                        <div class=styles::spinner></div>
                    </div>
                </Show>

                <div class=styles::answer_reveal_section>
                    {move || {
                        let state = game_state.get();
                        if state == GameState::Win || state == GameState::Lose {
                            let (status_text, status_class) = match state {
                                GameState::Win => (texts().3, styles::status_win),
                                GameState::Lose => (texts().4, styles::status_lose),
                                _ => ("", ""),
                            };

                            view! {
                                <div>
                                    <div class=styles::reveal_container>
                                        <h2 class=status_class>{move || status_text}</h2>
                                        <h4 class=status_class>
                                            {move || format!("{}/{}", guess_time.get(), current_config.get().max_guess)}
                                        </h4>
                                        <h4 class=status_class>Time: {formatted_time}</h4>
                                        <button
                                            class=styles::reset_btn
                                            on:click=reset_game
                                        >
                                            {reset_icon}
                                        </button>
                                        <hr class=styles::divider />
                                        <p class=styles::reveal_text> {move || texts().5} </p>

                                        <Suspense fallback=|| view! { "..." }>
                                            {move || Suspend::new(async move {
                                                match answer.get() {
                                                    Some(a) => view! {<div> <Card2 info=a.0.clone() comparison=a.1/> </div>},
                                                    None => view! { <div><span></span> </div> }
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
            </main>
        </ErrorBoundary>
    }
}

pub fn use_interval<T, F>(interval_millis: T, f: F)
where
    F: Fn() + Clone + 'static,
    T: Into<Signal<u64>> + 'static,
{
    let interval_millis = interval_millis.into();

    Effect::new(move |_| {
        let handle =
            set_interval_with_handle(f.clone(), Duration::from_millis(interval_millis.get()))
                .expect("could not create interval");

        on_cleanup(move || {
            handle.clear();
        });
    });
}
