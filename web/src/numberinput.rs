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
    pub oninput: Callback<InputData>,
}

pub struct NumberInput {
    _link: ComponentLink<Self>,
    props: NumberInputProps,
}

impl Component for NumberInput {
    type Message = Msg;
    type Properties = NumberInputProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { _link: link, props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                { self.props.label.clone() }
                <p>
                    <input
                        value=self.props.value.clone()
                        min=self.props.min.to_string()
                        class="input card-footer-item"
                        disabled=self.props.disabled
                        oninput=self.props.oninput.clone()
                        type="number" />
                </p>
            </div>
        }
    }
}
