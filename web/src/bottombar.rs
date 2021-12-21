use yew::{html, Children, Component, Context, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct BottomBarProps {
    #[prop_or_default]
    pub children: Children,
}

pub struct BottomBar;

impl BottomBar {
    pub fn item_class() -> String {
        "navbar-item is-expanded is-block has-text-centered".into()
    }
}

impl Component for BottomBar {
    type Message = ();
    type Properties = BottomBarProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <nav class="navbar is-link is-fixed-bottom is-block" role="navigation">
                <div class="navbar-brand">
                    { for ctx.props().children.iter() }
                </div>
            </nav>
        }
    }
}
