use crate::error::Error;
use crate::icon::Icon;
use crate::input::{Input, InputKind};
use gloo::storage::{LocalStorage, Storage};
use gloo_timers::callback::Interval;
use pomododragon::{
    Actor, InstantTimer, Pomo, PomoActions, PomoCommand, PomoData, PomoMessage, PomoState,
    SimplePomo, SimpleTask, TimeParser, Timer,
};
use std::time::Duration;
use yew::prelude::*;

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
    _task: Interval,
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

    fn create(ctx: &Context<Self>) -> Self {
        let pomo = SimplePomo::default();
        let link = ctx.link().clone();

        let mut n = Self {
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
            state: TabState::Timer,
            _task: Interval::new(200, move || {
                link.send_message(Msg::Tick);
            }),
        };

        n.update(ctx, Msg::UpdateWorkTime(n.work_time_buffer.clone()));
        n.update(
            ctx,
            Msg::UpdateShortBreakTime(n.short_break_time_buffer.clone()),
        );
        n.update(
            ctx,
            Msg::UpdateLongBreakTime(n.long_break_time_buffer.clone()),
        );
        n.update(ctx, Msg::UpdateTotalCycles(n.total_cycles_buffer.clone()));
        n.update(
            ctx,
            Msg::UpdateUntilLongBreak(n.until_long_break_buffer.clone()),
        );

        let tasks: Vec<String> = LocalStorage::get(TASKS_KEY).unwrap_or_else(|_| vec![]);
        for task in tasks {
            // this usually will not fail!
            n.pomo
                .execute(PomoCommand::AddTask(SimpleTask::new(&task)))
                .expect("AddTaskFailed");
        }

        n
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Start => {
                if self.pomo.start().is_err() {
                    self.update(ctx, Msg::Error(Error::Start));
                }
                true
            }
            Msg::Pause => {
                if self.pomo.pause().is_err() {
                    self.update(ctx, Msg::Error(Error::Pause));
                }
                true
            }
            Msg::Resume => {
                if self.pomo.unpause().is_err() {
                    self.update(ctx, Msg::Error(Error::Unpause));
                }
                true
            }
            Msg::Stop => {
                if self.pomo.reset().is_err() {
                    self.update(ctx, Msg::Error(Error::Reset));
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
                        self.update(ctx, Msg::Error(Error::AddTask));
                    }

                    self.store_tasks(ctx);

                    self.description_buffer = "".into();
                }
                true
            }
            Msg::Delete(index) => {
                if self.pomo.execute(PomoCommand::RemoveTask(index)).is_err() {
                    self.update(ctx, Msg::Error(Error::LocalStorageWrite));
                }
                self.store_tasks(ctx);
                true
            }
            Msg::Update(value) => {
                self.description_buffer = value;
                true
            }
            Msg::UpdateWorkTime(value) => {
                if LocalStorage::set(WORK_TIME_KEY, value.clone()).is_err() {
                    self.update(ctx, Msg::Error(Error::LocalStorageWrite));
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
                    self.update(ctx, Msg::Error(Error::LocalStorageWrite));
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
                    self.update(ctx, Msg::Update("LocalStorageWriteFailed".into()));
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
                    self.update(ctx, Msg::Error(Error::LocalStorageWrite));
                }
                self.until_long_break_buffer = value;
                self.pomo.cycles_until_long_break =
                    self.total_cycles_buffer.parse::<usize>().unwrap_or(8);
                true
            }
            Msg::UpdateTotalCycles(value) => {
                if LocalStorage::set(TOTAL_CYCLES_KEY, value.clone()).is_err() {
                    self.update(ctx, Msg::Error(Error::LocalStorageWrite));
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
                    self.store_tasks(ctx);
                }

                true
            }
            Msg::SetTab(tab) => {
                self.state = tab;
                true
            }
            Msg::SkipTo(state) => {
                if self.pomo.skip_to(state).is_err() {
                    self.update(ctx, Msg::Error(Error::Update));
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
                    self.update(ctx, Msg::PomoMessage(message))
                }
                Err(_) => self.update(ctx, Msg::Error(Error::Update)),
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <section class="section">
                <div class="container">
                    <div>
                        <div class="tabs">
                            <ul>
                                <li class={self.get_tab_active(TabState::Timer)}>
                                    <a onclick={ctx.link().callback(|_| Msg::SetTab(TabState::Timer))}>
                                        { "Tasks" }
                                    </a>
                                </li>
                                <li class={self.get_tab_active(TabState::Tasks)}>
                                    <a onclick={ctx.link().callback(|_| Msg::SetTab(TabState::Tasks))}>
                                        { "Tasks" }
                                    </a>
                                </li>
                                <li class={self.get_tab_active(TabState::Settings)}>
                                    <a onclick={ctx.link().callback(|_| Msg::SetTab(TabState::Settings))}>
                                        { "Settings" }
                                    </a>
                                </li>
                            </ul>
                        </div>
                        {
                            match self.state {
                                TabState::Settings => self.view_settings(ctx),
                                TabState::Tasks => self.view_task_list(ctx),
                                _ => self.view_timer(ctx)
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

    fn store_tasks(&mut self, ctx: &Context<Self>) {
        // collect task strings and push to local storage
        let tasks = self
            .pomo
            .tasks
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();
        if LocalStorage::set(TASKS_KEY, tasks).is_err() {
            self.update(ctx, Msg::Error(Error::LocalStorageWrite));
        }
    }

    fn is_timer_running(&self) -> bool {
        if let Some(timer) = self.pomo.timer() {
            timer.elapsed() != None
        } else {
            self.pomo.is_paused()
        }
    }

    fn view_timer(&self, ctx: &Context<Self>) -> Html {
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
                        value={self.progress.clone()}
                        max={self.goal.clone()}>
                    </progress>
                </div>
                <div class="box">
                { self.view_controls(ctx) }
                { self.view_state_skips(ctx) }
                </div>
            </div>
        }
    }

    fn view_start_stop(&self, ctx: &Context<Self>) -> Html {
        {
            // change start/stop buttom depending on the state
            if self.pomo.state() != PomoState::NotStarted {
                html! {
                    <button
                        class="button is-warning"
                        disabled={ self.pomo.is_paused() }
                        onclick={ctx.link().callback(|_| Msg::Stop)}>
                        <Icon class={"fas fa-stop"} alt={"Stop"} />
                    </button>
                }
            } else {
                html! {
                    <button
                        class="button is-primary"
                        disabled={ self.pomo.is_paused() }
                        onclick={ctx.link().callback(|_| Msg::Start)}>
                        <Icon class={"fas fa-play"} alt={"Start"}/>
                    </button>
                }
            }
        }
    }

    fn view_pause_resume(&self, ctx: &Context<Self>) -> Html {
        if self.pomo.is_paused() {
            html! {
                <button
                    class="button is-info"
                    disabled={ self.pomo.state() == PomoState::NotStarted }
                    onclick={ctx.link().callback(|_| Msg::Resume)}>
                    <Icon class={"fas fa-play"} alt={"Resume"}/>
                </button>
            }
        } else {
            html! {
                <button
                    class="button is-info"
                    disabled={ self.pomo.state() == PomoState::NotStarted }
                    onclick={ctx.link().callback(|_| Msg::Pause)}>
                    <Icon class={"fas fa-pause"} alt={"Pause"}/>
                </button>
            }
        }
    }

    fn view_controls(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="buttons">
                { self.view_start_stop(ctx) }
                { self.view_pause_resume(ctx) }
            </div>
        }
    }

    fn view_state_skips(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="buttons">
                { self.view_skip_state("fas fa-briefcase", "Working", PomoState::Working, ctx) }
                { self.view_skip_state("fas fa-coffee", "Break", PomoState::Break, ctx) }
                { self.view_skip_state("fas fa-bed", "Long Break", PomoState::LongBreak, ctx) }
            </div>
        }
    }

    fn view_skip_state(
        &self,
        icon: &str,
        label: &str,
        state: PomoState,
        ctx: &Context<Self>,
    ) -> Html {
        html! {
            <button
                class="button is-info"
                disabled={ self.pomo.state() == PomoState::NotStarted }
                onclick={ctx.link().callback(move |_| Msg::SkipTo(state))}>
                <Icon class={icon.to_string()} alt={label.to_string()}/>
            </button>
        }
    }

    fn view_settings(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="content box">
                <article class="content">
                    <label>
                        <Input
                            value={self.work_time_buffer.clone()}
                            oninput={ctx.link().callback(
                                Msg::UpdateWorkTime)}
                            min={1}
                            disabled={self.is_timer_running()}
                            label="Work"
                            kind={InputKind::Number}
                        />
                    </label>
                    <label>
                        <Input
                            value={self.short_break_time_buffer.clone()}
                            oninput={ctx.link().callback(
                                Msg::UpdateShortBreakTime)}
                            min={1}
                            disabled={self.is_timer_running()}
                            label="Short Break"
                            kind={InputKind::Number}
                        />
                    </label>

                    <label>
                        <Input
                            value={self.long_break_time_buffer.clone()}
                            oninput={ctx.link().callback(
                                Msg::UpdateLongBreakTime)}
                            min={1}
                            disabled={self.is_timer_running()}
                            label="Long Break"
                            kind={InputKind::Number}
                        />
                    </label>

                    <label>
                        <Input
                            value={self.until_long_break_buffer.clone()}
                            oninput={ctx.link().callback(
                                Msg::UpdateUntilLongBreak)}
                            min={1}
                            label="Cycles Until Long Break"
                            kind={InputKind::Number}
                        />
                    </label>
                    <label>
                        <Input
                            value={self.total_cycles_buffer.clone()}
                            oninput={ctx.link().callback(
                                Msg::UpdateTotalCycles)}
                            min={1}
                            label="Total Cycles"
                            kind={InputKind::Number}
                        />
                    </label>
                </article>
            </div>
        }
    }

    fn view_task(&self, task: &SimpleTask, index: usize, ctx: &Context<Self>) -> Html {
        html! {
            <div class="message">
                <div class="message-header">
                    { task.to_string() }
                    <button
                        class="delete"
                        aria-label="delete"
                        onclick={ctx.link().callback(move |_| Msg::Delete(index))}>
                    </button>
                </div>
            </div>
        }
    }

    fn view_task_list(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="container box">
                <article class="content box">
                    <div class="columns">
                       <Input
                         class="column is-three-quarters"
                         input_class="input is-primary"
                         kind={InputKind::Text}
                         placeholder="What needs to be done?"
                         value={self.description_buffer.clone()}
                         oninput={ctx.link().callback(
                             Msg::Update)
                         }
                         onkeypress={ctx.link().batch_callback(|e: KeyboardEvent| {
                                 if e.key() == "Enter" { Some(Msg::Add) } else { None }
                             })}
                        />
                        <div class="column">
                            <button
                                class="button is-info"
                                onclick={ctx.link().callback(|_| Msg::Add)}>
                                { "Add" }
                            </button>
                        </div>
                    </div>
                </article>
                {
                    for self.pomo.tasks().iter()
                        .enumerate()
                        .map(|(i, task)| self.view_task(task, i,ctx))
                }
            </div>
        }
    }
}
