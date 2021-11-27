use yew::prelude::*;

pub enum Msg {}

pub struct NotFound {
    _link: ComponentLink<Self>,
}

impl Component for NotFound {
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
            <section class="section">
                <div class="content">
                    { "404 Page not found" }
                </div>
            </section>
        }
    }
}
