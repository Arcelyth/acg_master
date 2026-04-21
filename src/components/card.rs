use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/card.module.scss");

use crate::components::{lang_list::LangList, theme_btn::ThemeBtn};
use crate::items::Card;

#[component]
pub fn Card(
    card: Card,
) -> impl IntoView {
    
    view! {
        <div 
            class=styles::card
        >
            {card.name}
        </div>
    }
}
