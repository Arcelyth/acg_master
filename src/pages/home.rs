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
            "标题",
            "副标题",
            "动漫名称",
            "动漫人物",
        ),
        Language::English => (
            "AnimeMaster",
            "omg!",
            "Anime Name",
            "Anime Character",
        ),
    };
    let intro_text = move || match config.get().lang {
        Language::Chinese => (
            "说明：在特定次数内猜出动漫名称为获胜条件，对于某次猜测给出的信息，",
            " 代表该项与目标答案完全匹配；而",
            " 则说明该项与答案接近。"
        ),
        Language::English => (
            "Rules: In limited guess times, guess the anime name. For each guess, ",
            " indicates a perfect match; ",
            " indicates a close attribute."
        ),
    };

    view! {
        <main>
            <div class=styles::title_container>
                <h1 class=styles::main_title>{move || t().0}</h1>
                <p class=styles::sub_title>{move || t().1}</p>
            </div>

            <div class=styles::button_row>
                <JmpBtn text={move || t().2.to_string()} url="/single".to_string()/>
                <JmpBtn text={move || t().3.to_string()} url="/single2".to_string()/>
            </div>

            <div class=styles::rules_container>
               <div class=styles::rules> 
                    <p>
                        {move || intro_text().0}
                        <span style="     
                              display: inline-block;
                              width: 1.2rem;
                              height: 1.2rem;
                              border-radius: 0.25rem;
                              vertical-align: middle;
                              margin: 0 0.3rem;
                              background-color: #eef8f2;
                              color: #277b4c;
                              border: 0.0625rem solid #d1ebd8;
                        "> </span>
                        {move || intro_text().1}
                        <span style="
                              display: inline-block;
                              width: 1.2rem;
                              height: 1.2rem;
                              border-radius: 0.25rem;
                              vertical-align: middle;
                              margin: 0 0.3rem;
                              background-color: #fff9ed;
                              color: #9c6c19;
                              border: 0.0625rem solid #f7e6ca;
                        "> </span>
                        {move || intro_text().2}
                    </p>
               </div>
            </div>
        </main>
    }
}
