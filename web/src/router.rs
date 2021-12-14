use crate::about::About;
use crate::footer::Footer;
use crate::nav::Nav;
use crate::notfound::NotFound;
use crate::App;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Routable, Debug, Clone, PartialEq)]
pub enum AppRoute {
    #[at("/")]
    Index,

    #[at("/about")]
    About,

    #[not_found]
    #[at("/404")]
    NotFound,
}

pub enum Msg {}

pub struct AppRouter {}

impl Component for AppRouter {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <Nav />
                <BrowserRouter>
                    <Switch<AppRoute>
                    render = {
                        Switch::render(|switch: &AppRoute| {
                            match switch {
                                AppRoute::Index => html!{<App />},
                                AppRoute::About => html!{<About />},
                                _ => html!{<NotFound />},
                            }
                        })
                    }
                    />
                </BrowserRouter>
                <Footer />
            </div>
        }
    }
}
