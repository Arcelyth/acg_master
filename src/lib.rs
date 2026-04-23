use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};
use stylance::import_style;

mod bangumi;
mod components;
mod config;
mod items;
mod pages;

use crate::components::{footer, header};
use crate::config::Config;
use crate::pages::{home::Home, not_found::NotFound, single::Single};

import_style!(styles, "app.module.scss");

#[component]
pub fn App() -> impl IntoView {
    let (config, set_config) = signal(Config::new());

    provide_meta_context();
    provide_context(set_config);
    provide_context(config);

    view! {
        <Html attr:lang="en" attr:dir="ltr" attr:data-theme=move || if config.get().theme_dark {"dark"} else {"light"} />
        <Title text="Anime Master" />
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <Router>
            <header::Header/>
            <Routes fallback=|| view! { NotFound }>
                <Route path=path!("/") view=Home />
                <Route path=path!("/single") view=Single />
                <Route path=path!("/*any") view=NotFound />
            </Routes>
            <footer::Footer/>
        </Router>
    }
}
