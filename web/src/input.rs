use web_sys::HtmlInputElement as InputElement;
use yew::prelude::*;
use yew::Properties;

pub enum Msg {}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InputKind {
    Text,
    Number,
}

#[derive(Properties, PartialEq, Clone)]
pub struct InputProps {
    #[prop_or_default]
    pub label: String,
    #[prop_or_default]
    pub value: String,

    #[prop_or("input".into())]
    pub input_class: String,

    #[prop_or_default]
    pub class: String,

    #[prop_or_default]
    pub placeholder: String,

    #[prop_or(0)]
    pub min: i64,
    #[prop_or(std::i64::MAX)]
    pub max: i64,

    #[prop_or_default]
    pub disabled: bool,

    #[prop_or(InputKind::Text)]
    pub kind: InputKind,

    #[prop_or_default]
    pub oninput: Callback<String>,

    #[prop_or_default]
    pub onkeypress: Callback<KeyboardEvent>,
}

pub struct Input {}

impl Component for Input {
    type Message = Msg;
    type Properties = InputProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback = ctx.props().oninput.clone();
        html! {
            <div class={ctx.props().class.clone()}>
                { ctx.props().label.clone() }
                <p>
                    <input
                        class={ctx.props().input_class.clone()}
                        placeholder={ctx.props().placeholder.clone()}
                        value={ctx.props().value.clone()}
                        min={ctx.props().min.to_string()}
                        disabled={ctx.props().disabled}
                        oninput={move |input: InputEvent| Self::on_input(input, &callback)}
                        onkeypress={ctx.props().onkeypress.clone()}
                        type={Self::input_kind(ctx)} />
                </p>
            </div>
        }
    }
}

impl Input {
    fn on_input(e: InputEvent, callback: &Callback<String>) {
        // TODO maybe don't trust this unchecked cast!
        let input: InputElement = e.target_unchecked_into();
        callback.emit(input.value());
    }

    fn input_kind(ctx: &Context<Self>) -> String {
        match ctx.props().kind {
            InputKind::Text => "text",
            InputKind::Number => "number",
        }
        .into()
    }
}
