use gloo::storage::{LocalStorage, Storage};
use pomododragon::{
    InstantTimer, Pomo, PomoMessage, PomoState, SimplePomo, SimpleTask, TimeParser, Timer,
};
use std::time::Duration;
use yew::prelude::*;
use yew::services::interval::{IntervalService, IntervalTask};
use yew::InputData;

// keys for local storage
const WORK_TIME_KEY: &str = "pomododragon.work_timer";
const BREAK_TIME_KEY: &str = "pomododragon.break_time";
const LONG_BREAK_TIME_KEY: &str = "pomododragon.long_break_time";
const TOTAL_CYCLES_KEY: &str = "pomododragon.total_cycles";
const CYCLES_UNTIL_BREAK_KEY: &str = "pomododragon.cycles_until_break";
const TASKS_KEY: &str = "pomododragon.tasks";

pub enum Msg {
    Start,
    Stop,
    Pause,
    Resume,
    Add,
    Delete(usize),
    Update(String),
    UpdateWorkTime(String),
    UpdateShortBreakTime(String),
    UpdateLongBreakTime(String),
    UpdateUntilLongBreak(String),
    UpdateTotalCycles(String),
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
    work_time_buffer: String,
    until_long_break_buffer: String,
    total_cycles_buffer: String,
    progress: String,
    goal: String,
    short_break_time_buffer: String,
    long_break_time_buffer: String,
    _task: IntervalTask,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|_| Msg::Tick);
        let task = IntervalService::spawn(Duration::from_millis(200), callback);
        let pomo = SimplePomo::default();

        let mut n = Self {
            link,
            pomo,
            description_buffer: "".into(),
            work_time_buffer: LocalStorage::get(WORK_TIME_KEY).unwrap_or_else(|_| "25".into()),
            short_break_time_buffer: LocalStorage::get(BREAK_TIME_KEY)
                .unwrap_or_else(|_| "5".into()),
            long_break_time_buffer: LocalStorage::get(LONG_BREAK_TIME_KEY)
                .unwrap_or_else(|_| "30".into()),
            progress: "0".into(),
            goal: "100".into(),
            until_long_break_buffer: LocalStorage::get(CYCLES_UNTIL_BREAK_KEY)
                .unwrap_or_else(|_| "4".into()),
            total_cycles_buffer: LocalStorage::get(TOTAL_CYCLES_KEY).unwrap_or_else(|_| "8".into()),
            _task: task,
        };

        n.update(Msg::UpdateWorkTime(n.work_time_buffer.clone()));
        n.update(Msg::UpdateShortBreakTime(n.short_break_time_buffer.clone()));
        n.update(Msg::UpdateLongBreakTime(n.long_break_time_buffer.clone()));
        n.update(Msg::UpdateTotalCycles(n.total_cycles_buffer.clone()));
        n.update(Msg::UpdateUntilLongBreak(n.until_long_break_buffer.clone()));

        let tasks: Vec<String> = LocalStorage::get(TASKS_KEY).unwrap_or_else(|_| vec![]);
        for task in tasks {
            n.pomo.tasks.push(SimpleTask::new(&task));
        }

        n
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Start => {
                // TODO don't unwrap!
                self.pomo.start().expect("Unable to unpause");
                true
            }
            Msg::Pause => {
                self.pomo.pause().expect("Unable to pause");
                true
            }
            Msg::Resume => {
                self.pomo.unpause().expect("Unable to unpause");
                true
            }
            Msg::Stop => {
                self.pomo.reset().expect("Reset failed");
                true
            }
            Msg::Add => {
                if !self.description_buffer.is_empty() {
                    self.pomo
                        .tasks
                        .push(SimpleTask::new(&self.description_buffer));

                    self.store_tasks();

                    self.description_buffer = "".into();
                }
                true
            }
            Msg::Delete(index) => {
                self.pomo.tasks.remove(index);
                self.store_tasks();
                true
            }
            Msg::Update(value) => {
                self.description_buffer = value;
                true
            }
            Msg::UpdateWorkTime(value) => {
                LocalStorage::set(WORK_TIME_KEY, value.clone())
                    .expect("Unable to write to local storage!");

                self.work_time_buffer = value;
                self.pomo.work_timer = InstantTimer::new(
                    TimeParser::parse(&format!("{}m", self.work_time_buffer))
                        .unwrap_or_else(|| Duration::from_secs(0)),
                );

                true
            }
            Msg::UpdateShortBreakTime(value) => {
                LocalStorage::set(BREAK_TIME_KEY, value.clone())
                    .expect("Unable to write to local storage!");

                self.short_break_time_buffer = value;
                self.pomo.break_timer = InstantTimer::new(
                    TimeParser::parse(&format!("{}m", self.short_break_time_buffer))
                        .unwrap_or_else(|| Duration::from_secs(0)),
                );
                true
            }
            Msg::UpdateLongBreakTime(value) => {
                LocalStorage::set(LONG_BREAK_TIME_KEY, value.clone())
                    .expect("Unable to write to local storage!");

                self.long_break_time_buffer = value;
                self.pomo.long_break_timer = InstantTimer::new(
                    TimeParser::parse(&format!("{}m", self.long_break_time_buffer))
                        .unwrap_or_else(|| Duration::from_secs(0)),
                );
                true
            }
            Msg::UpdateUntilLongBreak(value) => {
                LocalStorage::set(CYCLES_UNTIL_BREAK_KEY, value.clone())
                    .expect("Unable to write to local storage!");
                self.until_long_break_buffer = value;
                self.pomo.cycles_until_long_break =
                    self.total_cycles_buffer.parse::<usize>().unwrap_or(8);
                true
            }
            Msg::UpdateTotalCycles(value) => {
                LocalStorage::set(TOTAL_CYCLES_KEY, value.clone())
                    .expect("Unable to write to local storage!");
                self.total_cycles_buffer = value;
                self.pomo.total_cycles = self.total_cycles_buffer.parse::<usize>().unwrap_or(8);
                true
            }
            Msg::Error(msg) => {
                log::error!("{}", msg);
                true
            }
            Msg::PomoMessage(message) => {
                if let PomoMessage::Transition(_) = message {
                    self.store_tasks();
                }

                true
            }
            Msg::Tick => match self.pomo.update() {
                Ok(message) => {
                    if let Some(timer) = self.pomo.timer() {
                        self.progress = format!(
                            "{}",
                            timer
                                .elapsed()
                                .unwrap_or_else(|| Duration::from_secs(0))
                                .as_secs()
                        );
                        self.goal = format!("{}", timer.goal().as_secs());
                    }
                    self.update(Msg::PomoMessage(message))
                }
                Err(_) => self.update(Msg::Error("Unable to update!".into())),
            },
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <section class="section">
                <div class="container">
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
                </div>
            </section>
        }
    }
}

impl App {
    fn store_tasks(&self) {
        // collect task strings and push to local storage
        let tasks = self
            .pomo
            .tasks
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();
        LocalStorage::set(TASKS_KEY, tasks).expect("Unable to store in local storage!");
    }

    fn is_timer_running(&self) -> bool {
        if let Some(timer) = self.pomo.timer() {
            timer.elapsed() != None
        } else {
            self.pomo.is_paused()
        }
    }

    fn view_timer(&self) -> Html {
        html! {
            <div class="card">
                <div class="card-header">
                    <div class="card-header-title title">
                        { self.pomo.state() }
                    </div>
                </div>
                <div class="card-content">
                    <div class="content title">
                        {
                            if let Some(task) = self.pomo.task() {
                                task.to_string()
                            } else {
                                "".into()
                            }
                        }
                    </div>
                    <div class="content title">
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
                    <progress
                        class="progress is-primary is-large"
                        value=self.progress.clone()
                        max=self.goal.clone()>
                    </progress>
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
                        onclick=self.link.callback(|_| Msg::Stop)>
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

    fn view_pause_resume(&self) -> Html {
        if self.pomo.is_paused() {
            html! {
                <button
                    class="button is-primary card-footer-item"
                    onclick=self.link.callback(|_| Msg::Resume)>
                    { "Resume" }
                </button>
            }
        } else {
            html! {
                <button
                    class="button is-primary card-footer-item"
                    onclick=self.link.callback(|_| Msg::Pause)>
                    { "Pause" }
                </button>
            }
        }
    }

    fn view_controls(&self) -> Html {
        html! {
            <div class="card-footer">
                { self.view_start_stop() }
                { self.view_pause_resume() }
            </div>
        }
    }

    fn view_settings(&self) -> Html {
        html! {
            <div class="content">
                <article class="content">
                    <label>
                        { "Work" }
                        <input
                            value=self.work_time_buffer.clone()
                            oninput=self.link.callback(|e: InputData| Msg::UpdateWorkTime(e.value))
                            min="0"
                            class="input card-footer-item"
                            disabled=self.is_timer_running()
                            type="number" />
                    </label>
                    <label>
                        { "Short Break" }
                        <input
                            value=self.short_break_time_buffer.clone()
                            oninput=self.link.callback(|e: InputData| Msg::UpdateShortBreakTime(e.value))
                            min="0"
                            class="input card-footer-item"
                            disabled=self.is_timer_running()
                            type="number" />
                    </label>

                    <label>
                        { "Long Break" }
                        <input
                            value=self.long_break_time_buffer.clone()
                            oninput=self.link.callback(|e: InputData| Msg::UpdateLongBreakTime(e.value))
                            min="0"
                            class="input card-footer-item"
                            disabled=self.is_timer_running()
                            type="number" />
                    </label>

                    <label>
                        { "Cycles Until Long Break" }
                        <input
                            value=self.until_long_break_buffer.clone()
                            oninput=self.link.callback(|e: InputData| Msg::UpdateUntilLongBreak(e.value))
                            min="0"
                            class="input card-footer-item"
                            type="number" />
                    </label>
                    <label>
                        { "Total Cycles" }
                        <input
                            value=self.total_cycles_buffer.clone()
                            oninput=self.link.callback(|e: InputData| Msg::UpdateTotalCycles(e.value))
                            min="0"
                            class="input card-footer-item"
                            type="number" />
                    </label>
                </article>
                <article class="content">
                    <div class="columns">
                       <input
                         class="input is-primary column is-three-quarters"
                         type="text"
                         placeholder="What needs to be done?"
                         value=self.description_buffer.clone()
                         oninput=self.link.callback(|e: InputData| Msg::Update(e.value))
                         onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                                 if e.key() == "Enter" { Some(Msg::Add) } else { None }
                             })
                        />
                        <button
                            class="button is-primary column"
                            onclick=self.link.callback(|_| Msg::Add)>
                            { "Add" }
                        </button>
                    </div>
                </article>
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
