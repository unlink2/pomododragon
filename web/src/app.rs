use crate::footer::Footer;
use crate::nav::Nav;
use pomododragon::{
    InstantTimer, Pomo, PomoMessage, PomoState, SimplePomo, SimplePomoBuilder, SimpleTask, Task,
    TaskKind, Timer,
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
    Delete(usize),
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
            Msg::Delete(index) => {
                self.pomo.tasks.remove(index);
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
            Msg::PomoMessage(_message) => true,
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
            <section class="section">
                <div class="container">
                    <Nav />
                    <div>
                        {
                            self.view_timer()
                        }
                        {
                            self.view_settings()
                        }
                        {
                            self.view_task_list()
                        }
                    </div>
                    <Footer />
                </div>
            </section>
        }
    }
}

impl App {
    fn view_timer(&self) -> Html {
        html! {
            <div class="card">
                <div class="card-header">
                    <div class="card-header-title">
                        { self.pomo.state() }
                    </div>
                </div>
                <div class="card-content">
                    <div class="content">
                        {
                            if let Some(task) = self.pomo.task() {
                                task.to_string()
                            } else {
                                "".into()
                            }
                        }
                    </div>
                    <div class="content">
                        {
                            if let Some(timer) = self.pomo.timer() {
                                if let Some(elapsed) = timer.elapsed() {
                                    let mins = elapsed.as_secs() / 60;
                                    let secs = elapsed.as_secs() - mins * 60;
                                    format!("{:02}:{:02}", mins, secs)
                                } else {
                                    "00:00".into()
                                }
                            } else {
                                "00:00".into()
                            }
                        }
                    </div>
                    <progress class="progress is-primary is-large" value="10" max="100"></progress>
                </div>
                { self.view_controls() }
            </div>
        }
    }

    fn view_start_stop(&self) -> Html {
        {
            // change start/stop buttom depending on the state
            if self.pomo.state() == PomoState::Working {
                html! {
                    <button
                        class="button is-primary card-footer-item"
                        onclick=self.link.callback(|_| Msg::Start)>
                        { "Stop" }
                    </button>
                }
            } else {
                html! {
                    <button
                        class="button is-primary card-footer-item"
                        onclick=self.link.callback(|_| Msg::Start)>
                        { "Start" }
                    </button>
                }
            }
        }
    }

    fn view_controls(&self) -> Html {
        html! {
            <div class="card-footer">
                { self.view_start_stop() }
                <button
                 class="button is-primary card-footer-item"
                 onclick=self.link.callback(|_| Msg::Start)>
                     { "Pause" }
                </button>
            </div>
        }
    }

    fn view_settings(&self) -> Html {
        html! {
            <div class="content">
                <label>
                    { "Work time" }
                    <input class="input card-footer-item" type="number" />
                </label>
                <label>
                    { "Break time" }
                    <input class="input card-footer-item" type="number" />
                </label>

                <label>
                    { "Long Break" }
                    <input class="input card-footer-item" type="number" />
                </label>

                <div class="">
                   <label>
                       { "Task" }
                       <input
                         class="input is-primary"
                         type="text"
                         placeholder="What needs to be done?"
                         value=self.description_buffer.clone()
                         oninput=self.link.callback(|e: InputData| Msg::Update(e.value))
                         onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                                 if e.key() == "Enter" { Some(Msg::Add) } else { None }
                             })
                        />
                    </label>
                    <button
                     class="button is-primary"
                     onclick=self.link.callback(|_| Msg::Add)>
                         { "Add" }
                    </button>
                </div>
            </div>
        }
    }

    fn view_task(&self, task: &SimpleTask, index: usize) -> Html {
        html! {
            <div class="message">
                <div class="message-header">
                    { task.to_string() }
                    <button
                        class="delete"
                        aria-label="delete"
                        onclick=self.link.callback(move |_| Msg::Delete(index))>
                    </button>
                </div>
            </div>
        }
    }

    fn view_task_list(&self) -> Html {
        html! {
            <div class="container">
                {
                    for self.pomo.tasks().iter()
                        .enumerate()
                        .map(|(i, task)| self.view_task(task, i))
                }
            </div>
        }
    }
}
