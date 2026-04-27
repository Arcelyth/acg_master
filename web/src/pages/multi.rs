use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashSet;
use std::time::Duration;

use stylance::import_crate_style;

import_crate_style!(styles, "./src/pages/styles/multi.module.scss");

use crate::bangumi::anime::*;
use crate::components::back_btn::BackBtn;
use crate::components::card::Card;
use crate::config::{Config, Language};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GameState {
    Lobby,  // before matching
    Matching,
    Loading,
    Playing,
    Win,
    Lose,
}

#[component]
pub fn Multi() -> impl IntoView {
    let (game_state, set_game_state) = signal(GameState::Lobby);
    let (username, set_username) = signal("".to_string());
    let config = use_context::<ReadSignal<Config>>().expect("reader");

    // timer
    let (elapsed_seconds, set_elapsed_seconds) = signal(0u64);
    let (is_timer_running, set_is_timer_running) = signal(false);
    let (current_config, set_current_config) = signal(config.get_untracked());


    let formatted_time = move || {
        let s = elapsed_seconds.get();
        format!("{:02}:{:02}", s / 60, s % 60)
    };

    use_interval(1000, move || {
        if is_timer_running.get() {
            set_elapsed_seconds.update(|sec| *sec += 1);
        }
    });


    let start_match = move |_| {
        let name_len = username.get().trim().len();
        if name_len < 1 || name_len > 20 {
            return;
        }
        set_game_state.set(GameState::Matching);
    };

    let texts = move || match config.get().lang {
        Language::Chinese => (
            "输入名称",
            "开始匹配",
            "匹配中......",
        ),
        Language::English => (
            "Input your name",
            "Start matching",
            "Matching..."
        ),
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
                    <BackBtn />
                </div>        

                <Show when=move || game_state.get() == GameState::Lobby || game_state.get() == GameState::Matching>
                    <div class=styles::lobby_section>
                        <input 
                            class=styles::username_input
                            placeholder=texts().0 
                            bind:value=(username, set_username)
                            disabled=move || game_state.get() == GameState::Matching
                        />
                        <button class=styles::match_btn on:click=start_match disabled=move || game_state.get() == GameState::Matching>
                            {move || if game_state.get() == GameState::Matching { texts().2 } else { texts().1 }}
                        </button>
                    </div>
                </Show>
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
    Effect::new(move |prev_handle: Option<IntervalHandle>| {
        if let Some(prev_handle) = prev_handle {
            prev_handle.clear();
        };

        set_interval_with_handle(f.clone(), Duration::from_millis(interval_millis.get()))
            .expect("could not create interval")
    });
}
