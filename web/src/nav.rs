use yew::prelude::*;

pub enum Msg {}

pub struct Nav {
    _link: ComponentLink<Self>,
}

impl Component for Nav {
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
          <div class="navbar">
                <div class="navbar-brand">
                    <a class="navbar-item" href="/">
                        <img alt="logo" src="" width="112" height="28" />
                    </a>

                    <a role="button"
                        class="navbar-burger"
                        aria-label="menu"
                        aria-expanded="false"
                        data-target="navbar">
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>
                </div>

                <div id="navbar" class="navbar-menu">
                    <div class="navbar-start">
                        <a href="/" class="navbar-item">{ "Home" }</a>
                    </div>
                </div>
                 <div class="navbar-item has-dropdown is-hoverable">
                    <a class="navbar-link">
                        { "More" }
                    </a>

                    <div class="navbar-dropdown">
                      <a href="/about" class="navbar-item">
                          { "About" }
                      </a>
                    </div>
                </div>
            </div>
        }
    }
}
