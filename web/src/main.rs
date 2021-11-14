mod app;
mod footer;
mod nav;
use app::App;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::trace!("Initializing..");
    yew::start_app::<App>();
}
