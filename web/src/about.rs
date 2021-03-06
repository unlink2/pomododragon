use yew::prelude::*;

pub enum Msg {}

pub struct About {}

impl Component for About {
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
              <div class="content has-text-centered">
                <p>
                    <strong>{ "Contact: " }</strong>
                    <a href="mailto:lukas@krickl.dev">{ "lukas@krickl.dev" }</a>
                </p>
              </div>
            </section>
        }
    }
}
