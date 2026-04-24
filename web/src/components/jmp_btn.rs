use leptos::prelude::*;
use leptos_router::components::A;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/jmp_btn.module.scss");

#[component]
pub fn JmpBtn(
    #[prop(into)]
    text: Signal<String>,
    url: String,
) -> impl IntoView {
    view! {
        <A
            href=url 
            attr:class=styles::jmp_btn
        >
            <span class:text>
                {move || text.get()}
            </span>
        </A>
    }
}
