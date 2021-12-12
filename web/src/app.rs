use crate::error::Error;
use crate::numberinput::NumberInput;
use gloo::storage::{LocalStorage, Storage};
use pomododragon::{
    Actor, InstantTimer, Pomo, PomoCommand, PomoMessage, PomoState, SimplePomo, SimpleTask,
    TimeParser, Timer,
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
    SkipTo(PomoState),
    Error(Error),
    SetTab(TabState),
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
    state: TabState,
    _task: IntervalTask,
}

#[derive(PartialEq, Eq)]
pub enum TabState {
    Timer,
    Tasks,
    Settings,
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
            state: TabState::Timer,
        };

        n.update(Msg::UpdateWorkTime(n.work_time_buffer.clone()));
        n.update(Msg::UpdateShortBreakTime(n.short_break_time_buffer.clone()));
        n.update(Msg::UpdateLongBreakTime(n.long_break_time_buffer.clone()));
        n.update(Msg::UpdateTotalCycles(n.total_cycles_buffer.clone()));
        n.update(Msg::UpdateUntilLongBreak(n.until_long_break_buffer.clone()));

        let tasks: Vec<String> = LocalStorage::get(TASKS_KEY).unwrap_or_else(|_| vec![]);
        for task in tasks {
            // this usually will not fail!
            n.pomo
                .execute(PomoCommand::AddTask(SimpleTask::new(&task)))
                .expect("AddTaskFailed");
        }

        n
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Start => {
                if self.pomo.start().is_err() {
                    self.update(Msg::Error(Error::Start));
                }
                true
            }
            Msg::Pause => {
                if self.pomo.pause().is_err() {
                    self.update(Msg::Error(Error::Pause));
                }
                true
            }
            Msg::Resume => {
                if self.pomo.unpause().is_err() {
                    self.update(Msg::Error(Error::Unpause));
                }
                true
            }
            Msg::Stop => {
                if self.pomo.reset().is_err() {
                    self.update(Msg::Error(Error::Reset));
                }
                true
            }
            Msg::Add => {
                if !self.description_buffer.is_empty() {
                    if self
                        .pomo
                        .execute(PomoCommand::AddTask(SimpleTask::new(
                            &self.description_buffer,
                        )))
                        .is_err()
                    {
                        self.update(Msg::Error(Error::AddTask));
                    }

                    self.store_tasks();

                    self.description_buffer = "".into();
                }
                true
            }
            Msg::Delete(index) => {
                if self.pomo.execute(PomoCommand::RemoveTask(index)).is_err() {
                    self.update(Msg::Error(Error::LocalStorageWrite));
                }
                self.store_tasks();
                true
            }
            Msg::Update(value) => {
                self.description_buffer = value;
                true
            }
            Msg::UpdateWorkTime(value) => {
                if LocalStorage::set(WORK_TIME_KEY, value.clone()).is_err() {
                    self.update(Msg::Error(Error::LocalStorageWrite));
                }

                self.work_time_buffer = value;
                self.pomo.work_timer = InstantTimer::new(
                    TimeParser::parse(&format!("{}m", self.work_time_buffer))
                        .unwrap_or_else(|| Duration::from_secs(0)),
                );

                true
            }
            Msg::UpdateShortBreakTime(value) => {
                if LocalStorage::set(BREAK_TIME_KEY, value.clone()).is_err() {
                    self.update(Msg::Error(Error::LocalStorageWrite));
                }

                self.short_break_time_buffer = value;
                self.pomo.break_timer = InstantTimer::new(
                    TimeParser::parse(&format!("{}m", self.short_break_time_buffer))
                        .unwrap_or_else(|| Duration::from_secs(0)),
                );
                true
            }
            Msg::UpdateLongBreakTime(value) => {
                if LocalStorage::set(LONG_BREAK_TIME_KEY, value.clone()).is_err() {
                    self.update(Msg::Update("LocalStorageWriteFailed".into()));
                }

                self.long_break_time_buffer = value;
                self.pomo.long_break_timer = InstantTimer::new(
                    TimeParser::parse(&format!("{}m", self.long_break_time_buffer))
                        .unwrap_or_else(|| Duration::from_secs(0)),
                );
                true
            }
            Msg::UpdateUntilLongBreak(value) => {
                if LocalStorage::set(CYCLES_UNTIL_BREAK_KEY, value.clone()).is_err() {
                    self.update(Msg::Error(Error::LocalStorageWrite));
                }
                self.until_long_break_buffer = value;
                self.pomo.cycles_until_long_break =
                    self.total_cycles_buffer.parse::<usize>().unwrap_or(8);
                true
            }
            Msg::UpdateTotalCycles(value) => {
                if LocalStorage::set(TOTAL_CYCLES_KEY, value.clone()).is_err() {
                    self.update(Msg::Error(Error::LocalStorageWrite));
                }
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
            Msg::SetTab(tab) => {
                self.state = tab;
                true
            }
            Msg::SkipTo(state) => {
                if self.pomo.skip_to(state).is_err() {
                    self.update(Msg::Error(Error::Update));
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
                Err(_) => self.update(Msg::Error(Error::Update)),
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
                        <div class="tabs">
                            <ul>
                                <li class={self.get_tab_active(TabState::Timer)}>
                                    <a onclick=self.link.callback(|_| Msg::SetTab(TabState::Timer))>
                                        { "Tasks" }
                                    </a>
                                </li>
                                <li class={self.get_tab_active(TabState::Tasks)}>
                                    <a onclick=self.link.callback(|_| Msg::SetTab(TabState::Tasks))>
                                        { "Tasks" }
                                    </a>
                                </li>
                                <li class={self.get_tab_active(TabState::Settings)}>
                                    <a onclick=self.link.callback(|_| Msg::SetTab(TabState::Settings))>
                                        { "Settings" }
                                    </a>
                                </li>
                            </ul>
                        </div>
                        {
                            match self.state {
                                TabState::Settings => self.view_settings(),
                                TabState::Tasks => self.view_task_list(),
                                _ => self.view_timer()
                            }
                        }
                    </div>
                </div>
            </section>
        }
    }
}

impl App {
    fn get_tab_active(&self, state: TabState) -> String {
        if state == self.state {
            "is-active".into()
        } else {
            "".into()
        }
    }

    fn store_tasks(&mut self) {
        // collect task strings and push to local storage
        let tasks = self
            .pomo
            .tasks
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();
        if LocalStorage::set(TASKS_KEY, tasks).is_err() {
            self.update(Msg::Error(Error::LocalStorageWrite));
        }
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
            <div class="container box is-primary">
                <div class="">
                    <div class="title">
                        { self.pomo.state() }
                    </div>
                </div>
                <div class="">
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
                { self.view_state_skips() }
            </div>
        }
    }

    fn view_start_stop(&self) -> Html {
        {
            // change start/stop buttom depending on the state
            if self.pomo.state() != PomoState::NotStarted {
                html! {
                    <button
                        class="button is-warning"
                        disabled={ self.pomo.is_paused() }
                        onclick=self.link.callback(|_| Msg::Stop)>
                        { "Stop" }
                    </button>
                }
            } else {
                html! {
                    <button
                        class="button is-primary"
                        disabled={ self.pomo.is_paused() }
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
                    class="button is-info"
                    disabled={ self.pomo.state() == PomoState::NotStarted }
                    onclick=self.link.callback(|_| Msg::Resume)>
                    { "Resume" }
                </button>
            }
        } else {
            html! {
                <button
                    class="button is-info"
                    disabled={ self.pomo.state() == PomoState::NotStarted }
                    onclick=self.link.callback(|_| Msg::Pause)>
                    { "Pause" }
                </button>
            }
        }
    }

    fn view_controls(&self) -> Html {
        html! {
            <div class="box">
                { self.view_start_stop() }
                { self.view_pause_resume() }
            </div>
        }
    }

    fn view_state_skips(&self) -> Html {
        html! {
            <div class="box">
                { self.view_skip_state("Working", PomoState::Working) }
                { self.view_skip_state("Break", PomoState::Break) }
                { self.view_skip_state("Long Break", PomoState::LongBreak) }
            </div>
        }
    }

    fn view_skip_state(&self, label: &str, state: PomoState) -> Html {
        html! {
            <button
                class="button is-info"
                disabled={ self.pomo.state() == PomoState::NotStarted }
                onclick=self.link.callback(move |_| Msg::SkipTo(state))>
                { label }
            </button>
        }
    }

    fn view_settings(&self) -> Html {
        html! {
            <div class="content box">
                <article class="content">
                    <label>
                        <NumberInput
                            value=self.work_time_buffer.clone()
                            oninput=self.link.callback(|e: InputData| Msg::UpdateWorkTime(e.value))
                            min={1}
                            disabled=self.is_timer_running()
                            label="Work"
                        />
                    </label>
                    <label>
                        <NumberInput
                            value=self.short_break_time_buffer.clone()
                            oninput=self.link.callback(|e: InputData| Msg::UpdateShortBreakTime(e.value))
                            min={1}
                            disabled=self.is_timer_running()
                            label="Short Break" />
                    </label>

                    <label>
                        <NumberInput
                            value=self.long_break_time_buffer.clone()
                            oninput=self.link.callback(|e: InputData| Msg::UpdateLongBreakTime(e.value))
                            min={1}
                            disabled=self.is_timer_running()
                            label="Long Break" />
                    </label>

                    <label>
                        <NumberInput
                            value=self.until_long_break_buffer.clone()
                            oninput=self.link.callback(|e: InputData| Msg::UpdateUntilLongBreak(e.value))
                            min={1}
                            label="Cycles Until Long Break" />
                    </label>
                    <label>
                        <NumberInput
                            value=self.total_cycles_buffer.clone()
                            oninput=self.link.callback(|e: InputData| Msg::UpdateTotalCycles(e.value))
                            min={1}
                            label="Total Cycles" />
                    </label>
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
            <div class="container box">
                <article class="content box">
                    <div class="columns">
                       <input
                         class="input is-primary is-three-quarters"
                         type="text"
                         placeholder="What needs to be done?"
                         value=self.description_buffer.clone()
                         oninput=self.link.callback(|e: InputData| Msg::Update(e.value))
                         onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                                 if e.key() == "Enter" { Some(Msg::Add) } else { None }
                             })
                        />
                        <button
                            class="button is-info"
                            onclick=self.link.callback(|_| Msg::Add)>
                            { "Add" }
                        </button>
                    </div>
                </article>
                {
                    for self.pomo.tasks().iter()
                        .enumerate()
                        .map(|(i, task)| self.view_task(task, i))
                }
            </div>
        }
    }
}
