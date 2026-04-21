use leptos::prelude::*;

use stylance::import_crate_style;

import_crate_style!(styles, "./src/pages/styles/home.module.scss");

use crate::components::jmp_btn::JmpBtn;
use crate::config::{Config, Language};

#[component]
pub fn Home() -> impl IntoView {
    let config_setter = use_context::<WriteSignal<Config>>().expect("setter");
    let config = use_context::<ReadSignal<Config>>().expect("reader");

    let play_text = move || {
        match config.get().lang {
            Language::Chinese => "开始",
            Language::English => "Start",
        }
        .to_string()
    };

    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <h1>"Uh oh! Something went wrong!"</h1>
                <p>"Errors: "</p>
                <ul>
                    {move || {
                        errors
                            .get()
                            .into_iter()
                            .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                            .collect_view()
                    }}

                </ul>
            }
        }>
            <div>Home</div>
            <JmpBtn text=play_text url="/single".to_string()/>
        </ErrorBoundary>
    }
}
