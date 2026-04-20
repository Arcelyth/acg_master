use leptos::prelude::*;
use anime_master::App;

fn main() {
    // set up logging
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! {
            <App />
        }
    })
}
