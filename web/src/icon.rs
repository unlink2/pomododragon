use yew::prelude::*;
use yew::Properties;

#[derive(Properties, PartialEq, Clone)]
pub struct IconProps {
    #[prop_or_default]
    pub class: String,
    #[prop_or_default]
    pub alt: String,
}

pub struct Icon;

impl Component for Icon {
    type Message = ();
    type Properties = IconProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <span class="icon is-small">
                <i class={ctx.props().class.clone()} aria-hidden="true" title={ctx.props().alt.clone()}></i>
                <span class="sr-only">{ctx.props().alt.clone()}</span>
            </span>
        }
    }
}
