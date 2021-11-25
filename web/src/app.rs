use crate::footer::Footer;
use crate::nav::Nav;
use pomododragon::{
    InstantTimer, Pomo, PomoMessage, PomoState, SimplePomo, SimplePomoBuilder, SimpleTask, Timer,
};
use std::time::Duration;
use yew::prelude::*;
use yew::services::interval::{IntervalService, IntervalTask};
use yew::InputData;

pub enum Msg {
    Start,
    Stop,
    Pause,
    Add,
    Update(String),
    PomoMessage(PomoMessage<SimpleTask, ()>),
    Error(String),
    Tick,
}

pub struct App {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    pomo: SimplePomo<SimpleTask, InstantTimer>,
    description_buffer: String,
    _task: IntervalTask,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|_| Msg::Tick);
        let task = IntervalService::spawn(Duration::from_millis(200), callback);
        let mut pomo = SimplePomo::default();

        // this actually cannot fail in this case!
        // pause immediatly to avoid ticking
        pomo.pause().expect("Unable to pause!");

        Self {
            link,
            pomo,
            description_buffer: "".into(),
            _task: task,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Start => {
                // TODO don't unwrap!
                self.pomo.unpause().expect("Unable to unpause");
                true
            }
            Msg::Add => {
                self.pomo
                    .tasks
                    .push(SimpleTask::new(&self.description_buffer));
                self.description_buffer = "".into();
                true
            }
            Msg::Update(value) => {
                self.description_buffer = value;
                true
            }
            Msg::Error(msg) => {
                log::error!("{}", msg);
                true
            }
            Msg::PomoMessage(message) => {
                log::info!("Got message {}", message);
                true
            }
            Msg::Tick => match self.pomo.update() {
                Ok(message) => self.update(Msg::PomoMessage(message)),
                Err(_) => self.update(Msg::Error("Unable to update!".into())),
            },
            _ => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container">
                <Nav />
                <div>
                    {
                        self.view_setup()
                    }
                    {
                        self.view_timer()
                    }
                    {
                        self.view_task_list()
                    }
                </div>
                <Footer />
            </div>
        }
    }
}

impl App {
    fn view_timer(&self) -> Html {
        html! {
            <div>
                { "Timer" }
            </div>
        }
    }

    fn view_setup(&self) -> Html {
        html! {
            <div>
               <input
                 class="new-todo"
                 placeholder="What needs to be done?"
                 value=self.description_buffer.clone()
                 oninput=self.link.callback(|e: InputData| Msg::Update(e.value))
                 onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                         if e.key() == "Enter" { Some(Msg::Add) } else { None }
                     })
                />
                <button
                 class="button is-primary"
                 onclick=self.link.callback(|_| Msg::Start)>
                     { "Start" }
                </button>
                <button
                 class="button is-primary"
                 onclick=self.link.callback(|_| Msg::Add)>
                     { "Add" }
                </button>
            </div>
        }
    }

    fn view_task_list(&self) -> Html {
        html! {
            <div>
            </div>
        }
    }
}
