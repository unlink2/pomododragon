use yew::prelude::*;
use yew::Properties;

pub enum Msg {}

#[derive(Properties, PartialEq, Clone)]
pub struct NumberInputProps {
    #[prop_or_default]
    pub label: String,
    #[prop_or_default]
    pub value: String,

    #[prop_or(0)]
    pub min: i64,
    #[prop_or(std::i64::MAX)]
    pub max: i64,

    #[prop_or_default]
    pub disabled: bool,

    #[prop_or_default]
    pub oninput: Callback<InputEvent>,
}

pub struct NumberInput {}

impl Component for NumberInput {
    type Message = Msg;
    type Properties = NumberInputProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                { ctx.props().label.clone() }
                <p>
                    <input
                        value={ctx.props().value.clone()}
                        min={ctx.props().min.to_string()}
                        class="input"
                        disabled={ctx.props().disabled}
                        oninput={ctx.props().oninput.clone()}
                        type="number" />
                </p>
            </div>
        }
    }
}
