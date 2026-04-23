use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/header.module.scss");

use crate::components::{lang_list::LangList, theme_btn::ThemeBtn};
use crate::config::{Config, Language};

#[component]
pub fn Header() -> impl IntoView {
    let (is_modal_open, set_is_modal_open) = signal(false);
    let (guess_count, set_guess_count) = signal(10usize);
    let set_config = use_context::<WriteSignal<Config>>().expect("setter");
    let config = use_context::<ReadSignal<Config>>().expect("reader");

    let texts = move || match config.get().lang {
        Language::Chinese => (
            "猜测次数",
            "重置以生效"
        ),
        Language::English => (
            "Guess Times",
            "Reset to apply"
        ),
    };

    view! {
        <div class=styles::header>
            <ThemeBtn />
            <LangList />

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
                    set_config.update(|v| v.max_guess = guess_count.get());
                        }>
                            <svg viewBox="0 0 24 24">
                                <line x1="18" y1="6" x2="6" y2="18"></line>
                                <line x1="6" y1="6" x2="18" y2="18"></line>
                            </svg>
                        </button>

                        <div class=styles::setting_item>
                            <label>{move || texts().0}</label>
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

                        <div class=styles::note_text>{move || texts().1}</div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
