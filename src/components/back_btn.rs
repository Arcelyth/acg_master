use leptos::prelude::*;
use leptos_router::components::A;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/back_btn.module.scss");

#[component]
pub fn BackBtn() -> impl IntoView {
    view! {
        <A
            href="/"
            attr:class=styles::back_btn
            attr:aria_label="Back to Home"
        >
            <svg 
                viewBox="0 0 24 24" 
                fill="none" 
                stroke="currentColor" 
                stroke-width="3" 
                stroke-linecap="round" 
                stroke-linejoin="round"
            >
                <polyline points="15 18 9 12 15 6"></polyline>
            </svg>
        </A>
    }
}
