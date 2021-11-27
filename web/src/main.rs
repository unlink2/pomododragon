mod about;
mod app;
mod footer;
mod nav;
mod notfound;
mod router;
use app::App;
use router::AppRouter;
use std::panic;

fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Initializing..");
    yew::start_app::<AppRouter>();
}
