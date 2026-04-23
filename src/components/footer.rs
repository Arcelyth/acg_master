use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/footer.module.scss");

#[component]
pub fn Footer() -> impl IntoView {

    view! {
        <div 
            class=styles::footer
        >
            Footer
        </div>
    }
}
