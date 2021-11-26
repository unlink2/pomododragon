use yew::prelude::*;

pub enum Msg {}

pub struct Footer {
    _link: ComponentLink<Self>,
}

impl Component for Footer {
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
            <footer class="footer">
              <div class="content has-text-centered">
                <p>
                    <strong>{ "Contact: " }</strong>
                    <a href="mailto:lukas@krickl.dev">{ "lukas@krickl.dev" }</a>
                </p>
              </div>
            </footer>
        }
    }
}
