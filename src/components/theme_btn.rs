
use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/theme_btn.module.scss");

use crate::config::Config;

#[component]
pub fn ThemeBtn() -> impl IntoView {
    let config_setter = use_context::<WriteSignal<Config>>().expect("setter");
    let config = use_context::<ReadSignal<Config>>().expect("reader");

    view! {
        <div
            class=move || {
                let base = styles::theme_btn.to_string();
                if config.get().theme_dark {
                    format!("{} {}", base, styles::dark)
                } else {
                    base
                }
            }
            on:click=move |_| {
                config_setter.update(|v| v.theme_dark = !v.theme_dark)
            }
        >
            <div class=styles::icon>
                <svg class=styles::sun viewBox="0 0 24 24">
                    // sun
                    <circle cx="12" cy="12" r="5"/>
                    <g>
                        <line x1="12" y1="1" x2="12" y2="4"/>
                        <line x1="12" y1="20" x2="12" y2="23"/>
                        <line x1="4.22" y1="4.22" x2="6.34" y2="6.34"/>
                        <line x1="17.66" y1="17.66" x2="19.78" y2="19.78"/>
                        <line x1="1" y1="12" x2="4" y2="12"/>
                        <line x1="20" y1="12" x2="23" y2="12"/>
                        <line x1="4.22" y1="19.78" x2="6.34" y2="17.66"/>
                        <line x1="17.66" y1="6.34" x2="19.78" y2="4.22"/>
                    </g>
                </svg>

                // moon
                <svg class=styles::moon viewBox="0 0 24 24">
                    <path d="M21 12.8A9 9 0 1 1 11.2 3 
                        7 7 0 0 0 21 12.8z"/>
                </svg>
            </div>
        </div>
    }
}
