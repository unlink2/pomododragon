use yew::prelude::*;

pub enum Msg {}

pub struct NotFound {}

impl Component for NotFound {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <section class="section">
                <div class="content">
                    { "404 Page not found" }
                </div>
            </section>
        }
    }
}
