use leptos::prelude::*;

use stylance::import_crate_style;

import_crate_style!(styles, "./src/pages/styles/single.module.scss");

#[component]
pub fn Single() -> impl IntoView {
    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <h1>"Uh oh! Something went wrong!"</h1>
                <p>"Errors: "</p>
                <ul>
                    {move || {
                        errors
                            .get()
                            .into_iter()
                            .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                            .collect_view()
                    }}

                </ul>
            }
        }>
           <div>Single Play</div>
           <a href="/">Home</a>
        </ErrorBoundary>
    }
}
