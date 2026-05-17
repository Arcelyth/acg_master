use leptos::prelude::*;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/components/styles/popup.module.scss");

#[component]
pub fn Popup(
    #[prop(into)] message: String,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <div class=styles::overlay on:click=move |_| on_close.run(())>
            <div class=styles::modal on:click=move |e| e.stop_propagation()>
                <div class=styles::content>
                    {message.clone()}
                </div>

                <button class=styles::close_btn on:click=move |_| on_close.run(())>
                    "关闭"
                </button>
            </div>
        </div>
    }
}
