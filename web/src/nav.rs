use yew::prelude::*;

pub enum Msg {
    ToggleMenu,
}

pub struct Nav {
    active: bool,
}

impl Nav {
    fn is_active(&self) -> String {
        if self.active { "is-active" } else { "" }.into()
    }
}

impl Component for Nav {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { active: false }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleMenu => self.active = !self.active,
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
          <div class="navbar is-primary">
                <div class="navbar-brand">
                    <a role="button"
                        onclick={ctx.link().callback(move |_| Msg::ToggleMenu)}
                        class="navbar-burger"
                        aria-label="menu"
                        aria-expanded="false"
                        data-target="navbar"
                        is-active={self.active.to_string()}
                        >
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>

                    <a class="navbar-item" href="/">
                        <img alt="logo" src="" width="112" height="28" />
                    </a>
                </div>

                <div id="navbar" class={classes!("navbar-menu", "has-text-light-link", self.is_active())}>
                    <div class="navbar-start">
                        <a href="/" class="navbar-item">{ "Home" }</a>
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
