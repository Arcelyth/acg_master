use leptos::prelude::*;
use acg_master_web::App;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! {
            <App />
        }
    })
}
