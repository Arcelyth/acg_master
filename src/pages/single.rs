use leptos::prelude::*;

use stylance::import_crate_style;

import_crate_style!(styles, "./src/pages/styles/single.module.scss");

use crate::components::jmp_btn::JmpBtn;
use crate::config::{Config, Language};

#[component]
pub fn Single() -> impl IntoView {
    let config_setter = use_context::<WriteSignal<Config>>().expect("setter");
    let config = use_context::<ReadSignal<Config>>().expect("reader");

    let return_text = move || {
        match config.get().lang {
            Language::Chinese => "返回",
            Language::English => "Back",
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
            <div 
                class=styles::top_section
            >
                <JmpBtn text=return_text url="/".to_string()/>
            </div>
            <div
                class=styles::interact_section
            >

            </div>
            <div
                class=styles::display_section
            >
            </div>
            <div
                class=styles::bottom_section
            >
            </div>
            
        </ErrorBoundary>
    }
}
