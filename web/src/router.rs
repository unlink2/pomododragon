use crate::about::About;
use crate::footer::Footer;
use crate::nav::Nav;
use crate::notfound::NotFound;
use crate::App;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Switch, Debug, Clone)]
pub enum AppRoute {
    #[to = "/!"]
    Index,

    #[to = "/about"]
    About,

    NotFound,
}

pub enum Msg {}

pub struct AppRouter {
    _link: ComponentLink<Self>,
}

impl Component for AppRouter {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { _link: link }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <Nav />
                <Router<AppRoute, ()>
                render = Router::render(|switch: AppRoute| {
                    match switch {
                        AppRoute::Index => html!{<App />},
                        AppRoute::About => html!{<About />},
                        _ => html!{<NotFound />},
                    }
                })
                />
                <Footer />
            </div>
        }
    }
}
