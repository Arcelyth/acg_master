use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/pages/styles/home.module.scss");

use crate::components::jmp_btn::JmpBtn;
use crate::config::{Config, Language};

#[component]
pub fn Home() -> impl IntoView {
    let config = use_context::<ReadSignal<Config>>().expect("reader");

    let t = move || match config.get().lang {
        Language::Chinese => (
            "ACG 高手",
            "🫵 来看看你是否是动漫膏手 🫵",
            "单人模式",
            "多人模式",
        ),
        Language::English => (
            "ACGMaster",
            "🫵 Let's see if you are an anime master 🫵",
            "Single",
            "Multi",
        ),
    };

    let intro_text = move || match config.get().lang {
        Language::Chinese => (
            "说明：在特定次数内猜出指定内容（比如:动漫名称)为获胜条件，对于某次猜测给出的信息，",
            " 代表该项与目标答案完全匹配；而",
            " 则说明该项与答案接近, 并且",
            " 的接近程度更高。",
        ),
        Language::English => (
            "Rules: In limited guess times, guess the anime name. For each guess, ",
            " indicates a perfect match; ",
            " indicates a close attribute.",
            " indicates that the item is very similar to the answer.",
        ),
    };

    view! {
        <main class=styles::main_layout>
            <div class=styles::title_container>
                <h1 class=styles::main_title>{move || t().0}</h1>
                <p class=styles::sub_title>{move || t().1}</p>
            </div>

            <div class=styles::button_row>
                <JmpBtn text={move || t().2.to_string()} url="/single".to_string()/>
                <JmpBtn text={move || t().3.to_string()} url="/multi".to_string()/>
            </div>

            <div class=styles::rules_container>
               <div class=styles::rules>
                    <p>
                        {move || intro_text().0}
                        <span class=styles::match_exact></span>
                        {move || intro_text().1}
                        <span class=styles::match_partial></span>
                        {move || intro_text().2}
                        <span class=styles::match_related></span>
                        {move || intro_text().3}
                    </p>
               </div>
            </div>
        </main>
    }
}
