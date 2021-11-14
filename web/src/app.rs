use crate::footer::Footer;
use crate::nav::Nav;
use pomododragon::{
    InstantTimer, Pomo, PomoState, SimplePomo, SimplePomoBuilder, SimpleTask, Timer,
};
use std::time::Duration;
use yew::prelude::*;
use yew::services::interval::{IntervalService, IntervalTask};

pub enum Msg {
    Start,
    Stop,
    Pause,
    Add,
    Tick,
}

pub struct App {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    pomo: Option<SimplePomo<SimpleTask, InstantTimer>>,
    _task: IntervalTask,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|_| Msg::Tick);
        let task = IntervalService::spawn(Duration::from_millis(200), callback);

        Self {
            link,
            pomo: None,
            _task: task,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Start => true,
            _ => true,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container">
                <Nav />
                    <button class="button is-primary" onclick=self.link.callback(|_| Msg::Start)>{ "Start" }</button>
                <Footer />
            </div>
        }
    }
}
