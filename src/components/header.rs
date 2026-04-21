use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/header.module.scss");

use crate::components::{lang_list::LangList, theme_btn::ThemeBtn};

#[component]
pub fn Header() -> impl IntoView {

    view! {
        <div 
            class=styles::header
        >
            <ThemeBtn />
            <LangList />
        </div>
    }
}
