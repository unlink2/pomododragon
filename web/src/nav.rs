use yew::prelude::*;

pub enum Msg {}

pub struct Nav {}

impl Component for Nav {
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
                    <a href="https://github.com/unlink2/pomododragon" class="navbar-item">
                          { "Source" }
                      </a>
                    </div>
                </div>
            </div>
        }
    }
}
