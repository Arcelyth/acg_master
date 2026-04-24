use leptos::prelude::*;
use stylance::import_crate_style;

use crate::config::{Config, Language};

import_crate_style!(styles, "./src/components/styles/lang_list.module.scss");

#[component]
pub fn LangList() -> impl IntoView {
    let config_setter =
        use_context::<WriteSignal<Config>>().expect("to have found the setter provided");
    let config = use_context::<ReadSignal<Config>>().expect("to have found the reader provided");

    let (is_open, is_open_setter) = signal(false);

    let current_lang_label = move || match config.get().lang {
        Language::Chinese => "简体中文",
        Language::English => "English",
    };

    let langs = vec![Language::Chinese, Language::English];

    view! {
        <div
            class=styles::dropdown
        >
            <div
                class=move || if is_open.get() {format!("{} {}", styles::drop_btn, styles::active)} else {styles::drop_btn.to_string()}
                on:click=move |_| is_open_setter.update(|v| *v = !*v)>
                {current_lang_label}
            </div>
            <ul
                class=move || if is_open.get() {format!("{} {}", styles::menu, styles::show)} else {styles::menu.to_string()}>
                {
                    langs.into_iter().map(|l| {
                        let label = match l {
                            Language::Chinese => "简体中文",
                            Language::English => "English",
                        };
                        view! {
                            <li
                                class=move || if config.get().lang as u8 == l as u8 {styles::selected} else {"".into()}
                                on:click=move |_| {
                                    config_setter.update(|c| c.lang = l);
                                    is_open_setter.set(false);
                            }>
                                {label}
                            </li>
                        }
                    }).collect_view()}
            </ul>
        </div>
    }
}
