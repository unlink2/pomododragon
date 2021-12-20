use crate::{Actor, PomoCommand, PomoMessage, Task, Timer, Transition};
use derive_builder::*;

/// Pomo is a simple state machine
/// with a timer and an output interface
/// The main communication between the application and
/// the pomo state machine should be done by using
/// the actor/command interface
///
/// All pomo machines should implement PomoMachine, PomoData and PomoActions as well as Actor
pub trait Pomo<TTask, TTimer>:
    Actor<PomoCommand<TTask>, Self::PomoOut> + PomoData<TTask, TTimer> + PomoActions<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    /// Either PomoMessage ot Result<PomoMessage, Error>
    type PomoOut;

    fn start(&mut self) -> Self::PomoOut;

    fn reset(&mut self) -> Self::PomoOut;
    fn clear(&mut self) -> Self::PomoOut;

    /// shoudl call output.update
    /// and output.task_compelted if a task was completed!
    fn update(&mut self) -> Self::PomoOut;
}

pub trait PomoData<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    fn state(&self) -> PomoState;

    /// returns the current timer
    fn timer(&self) -> Option<&TTimer>;

    /// returns the current task
    fn task(&self) -> Option<&TTask>;

    fn tasks(&self) -> &[TTask];
    fn tasks_mut(&mut self) -> &mut [TTask];

    fn is_paused(&self) -> bool {
        self.state() == PomoState::Paused
    }

    fn is_completed(&self) -> bool {
        self.state() == PomoState::Completed
    }
}

pub trait PomoActions<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    type PomoActionOut;

    /// sets the state
    fn set_state(&mut self, state: PomoState) -> Self::PomoActionOut;

    fn toggle_pause(&mut self) -> Self::PomoActionOut;
    fn pause(&mut self) -> Self::PomoActionOut;
    fn unpause(&mut self) -> Self::PomoActionOut;

    /// Stops all timers, skips to new state and starts apropriate timers
    /// should use set_state to start the apropriate state
    fn skip_to(&mut self, state: PomoState) -> Self::PomoActionOut;
}

/// All possible states a pomo machine can be in
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PomoState {
    NotStarted,
    Pending,
    Working,
    Break,
    LongBreak,
    Completed,
    Paused,
}

impl std::fmt::Display for PomoState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NotStarted => "NotStarted",
                Self::Pending => "Pending",
                Self::Working => "Working",
                Self::Break => "Break",
                Self::LongBreak => "Long Break",
                Self::Completed => "Completed",
                Self::Paused => "Paused",
            }
        )
    }
}

impl Default for PomoState {
    fn default() -> Self {
        Self::NotStarted
    }
}

/// A simple state machine
/// with a timer
#[derive(Builder, Debug)]
#[builder(setter(into))]
pub struct SimplePomo<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    #[builder(default)]
    pub tasks: Vec<TTask>,
    #[builder(default = "TTimer::default_break_timer()")]
    pub break_timer: TTimer, // short break
    #[builder(default = "TTimer::default_work_timer()")]
    pub work_timer: TTimer, // work
    #[builder(default = "TTimer::default_long_break_timer()")]
    pub long_break_timer: TTimer, // long break,

    // cycle counts
    #[builder(default = "0")]
    pub current_cycles: usize,
    #[builder(default = "4")]
    pub cycles_until_long_break: usize,
    #[builder(default = "8")]
    pub total_cycles: usize,

    #[builder(default = "PomoState::default()")]
    pub state: PomoState, // internal state
    #[builder(default = "PomoState::default()")]
    pub prev_state: PomoState,
}

impl<TTask, TTimer> Default for SimplePomo<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    fn default() -> Self {
        Self::new(
            vec![],
            TTimer::default_work_timer(),
            TTimer::default_break_timer(),
            TTimer::default_long_break_timer(),
        )
    }
}

impl<TTask, TTimer> SimplePomo<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    pub fn new(
        tasks: Vec<TTask>,
        work_timer: TTimer,
        break_timer: TTimer,
        long_break_timer: TTimer,
    ) -> Self {
        Self {
            tasks,
            work_timer,
            break_timer,
            long_break_timer,
            total_cycles: 8,
            cycles_until_long_break: 4,
            current_cycles: 0,
            state: PomoState::default(),
            prev_state: PomoState::default(),
        }
    }

    fn update_working(&mut self) -> PomoMessage<TTask> {
        // tick the timer
        if self.work_timer.is_completed() {
            self.current_cycles += 1;

            // remove first task and make it completed!
            let completed = if !self.tasks.is_empty() {
                let mut comp = self.tasks.remove(0);
                comp.complete();
                Some(comp)
            } else {
                None
            };

            // either long or regular break
            let mut msg = if self.current_cycles == self.total_cycles {
                // DONE!
                self.set_state(PomoState::Completed)
            } else if self.current_cycles % self.cycles_until_long_break == 0 {
                self.skip_to(PomoState::LongBreak)
            } else {
                self.skip_to(PomoState::Break)
            };
            // if we did transition, set the completed task
            if let PomoMessage::Transition(transition) = &mut msg {
                transition.completed = completed;
            }
            msg
        } else {
            PomoMessage::NoMessage
        }
    }

    fn update_break(&mut self) -> PomoMessage<TTask> {
        if self.break_timer.is_completed() {
            self.skip_to(PomoState::Working)
        } else {
            PomoMessage::NoMessage
        }
    }

    fn update_long_break(&mut self) -> PomoMessage<TTask> {
        if self.long_break_timer.is_completed() {
            self.skip_to(PomoState::Working)
        } else {
            PomoMessage::NoMessage
        }
    }
}

impl<TTask, TTimer> Actor<PomoCommand<TTask>, PomoMessage<TTask>> for SimplePomo<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    fn execute(&mut self, command: PomoCommand<TTask>) -> PomoMessage<TTask> {
        match command {
            PomoCommand::AddTask(task) => {
                self.tasks.push(task);
                PomoMessage::Executed
            }
            PomoCommand::RemoveTask(index) => {
                self.tasks.remove(index);
                PomoMessage::Executed
            }
            PomoCommand::Start => self.start(),
            PomoCommand::Pause => self.pause(),
            PomoCommand::Unpause => self.unpause(),
            PomoCommand::TogglePause => self.toggle_pause(),
            PomoCommand::Reset => self.reset(),
            PomoCommand::Update => self.update(),
            PomoCommand::Clear => self.clear(),
        }
    }
}

impl<TTask, TTimer> Pomo<TTask, TTimer> for SimplePomo<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    type PomoOut = PomoMessage<TTask>;

    fn start(&mut self) -> PomoMessage<TTask> {
        self.set_state(PomoState::Pending)
    }

    fn reset(&mut self) -> PomoMessage<TTask> {
        self.current_cycles = 0;
        self.state = PomoState::default();
        self.prev_state = PomoState::default();

        PomoMessage::Reset
    }

    fn clear(&mut self) -> PomoMessage<TTask> {
        self.tasks.clear();
        self.reset()
    }

    fn update(&mut self) -> PomoMessage<TTask> {
        match self.state() {
            PomoState::NotStarted => PomoMessage::NoMessage,
            PomoState::Pending => {
                // start the timer and change state
                self.work_timer.reset();
                self.set_state(PomoState::Working)
            }
            PomoState::Working => self.update_working(),
            PomoState::Break => self.update_break(),
            PomoState::LongBreak => self.update_long_break(),
            PomoState::Paused | PomoState::Completed => PomoMessage::NoMessage,
        }
    }
}

impl<TTask, TTimer> PomoActions<TTask, TTimer> for SimplePomo<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    type PomoActionOut = PomoMessage<TTask>;

    fn skip_to(&mut self, state: PomoState) -> PomoMessage<TTask> {
        self.work_timer.reset();
        self.break_timer.reset();
        self.long_break_timer.reset();

        match state {
            PomoState::Working => {
                self.work_timer.start();
            }
            PomoState::Break => {
                self.break_timer.start();
            }
            PomoState::LongBreak => {
                self.long_break_timer.start();
            }
            _ => (),
        };
        self.set_state(state)
    }

    /// Should call output.state_changed!
    fn set_state(&mut self, state: PomoState) -> PomoMessage<TTask> {
        self.prev_state = self.state();
        self.state = state;
        PomoMessage::Transition(Transition::new(self.prev_state, self.state()))
    }

    fn toggle_pause(&mut self) -> PomoMessage<TTask> {
        if !self.is_paused() {
            self.pause()
        } else {
            self.unpause()
        }
    }

    fn pause(&mut self) -> PomoMessage<TTask> {
        if !self.is_paused() {
            match self.state() {
                PomoState::Working => self.work_timer.pause(),
                PomoState::Break => self.break_timer.pause(),
                PomoState::LongBreak => self.long_break_timer.pause(),
                _ => (),
            }
            self.set_state(PomoState::Paused)
        } else {
            PomoMessage::Transition(Transition::new(PomoState::Paused, PomoState::Paused))
        }
    }

    fn unpause(&mut self) -> PomoMessage<TTask> {
        if self.is_paused() {
            match self.prev_state {
                PomoState::Working => self.work_timer.resume(),
                PomoState::Break => self.break_timer.resume(),
                PomoState::LongBreak => self.long_break_timer.resume(),
                _ => (),
            }
            self.set_state(self.prev_state)
        } else {
            PomoMessage::Transition(Transition::new(self.state, self.state))
        }
    }
}

impl<TTask, TTimer> PomoData<TTask, TTimer> for SimplePomo<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    fn task(&self) -> Option<&TTask> {
        self.tasks.get(0)
    }

    fn tasks(&self) -> &[TTask] {
        &self.tasks
    }

    fn tasks_mut(&mut self) -> &mut [TTask] {
        &mut self.tasks
    }

    fn timer(&self) -> Option<&TTimer> {
        match self.state() {
            PomoState::Working => Some(&self.work_timer),
            PomoState::Break => Some(&self.break_timer),
            PomoState::LongBreak => Some(&self.long_break_timer),
            _ => None,
        }
    }

    fn is_paused(&self) -> bool {
        self.state() == PomoState::Paused
    }

    fn state(&self) -> PomoState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{InstantTimer, SimpleTask};

    use super::*;

    #[test]
    fn it_should_update_states_in_order() {
        let bd = 100;
        let wd = 200;
        let rd = 250;
        let pd = 500;

        let mut pomo = SimplePomoBuilder::<SimpleTask, InstantTimer>::default()
            .break_timer(InstantTimer::new(Duration::from_millis(bd - 1)))
            .work_timer(InstantTimer::new(Duration::from_millis(wd - 1)))
            .long_break_timer(InstantTimer::new(Duration::from_millis(rd - 1)))
            .tasks(vec![
                SimpleTask::new("Task1".into()),
                SimpleTask::new("Task2".into()),
                SimpleTask::new("Task3".into()),
            ])
            .total_cycles(6_usize)
            .build()
            .unwrap();

        let output = pomo.start();
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::NotStarted, PomoState::Pending,))
        );

        // not started
        let output = pomo.update();

        // start working
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Pending, PomoState::Working,))
        );
        assert_eq!(pomo.task(), Some(&SimpleTask::new("Task1".into())));

        // *************
        // first update
        // *************
        let output = pomo.update();
        assert_eq!(pomo.task(), Some(&SimpleTask::new("Task1".into())));
        assert_eq!(output, PomoMessage::NoMessage);

        // *************
        // complete first work
        // *************
        std::thread::sleep(Duration::from_millis(wd));
        let output = pomo.update();
        let mut t1 = SimpleTask::new("Task1".into());
        // task completed call
        t1.complete();
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new_task(
                PomoState::Working,
                PomoState::Break,
                t1
            ))
        );

        // *************
        // update on break
        // *************
        let output = pomo.update();
        assert_eq!(output, PomoMessage::NoMessage);

        // *************
        // attempt pause
        // *************
        let output = pomo.toggle_pause();
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Break, PomoState::Paused))
        );

        assert!(pomo.break_timer.is_paused());
        assert!(!pomo.work_timer.is_paused());
        assert!(!pomo.long_break_timer.is_paused());

        // should still be paused!
        std::thread::sleep(Duration::from_millis(pd));
        let output = pomo.toggle_pause();
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Paused, PomoState::Break))
        );
        assert!(!pomo.break_timer.is_paused());
        assert!(!pomo.work_timer.is_paused());
        assert!(!pomo.long_break_timer.is_paused());

        let output = pomo.update();
        assert_eq!(pomo.task(), Some(&SimpleTask::new("Task2".into())));
        assert_eq!(output, PomoMessage::NoMessage);

        // *************
        // complete break
        // *************
        std::thread::sleep(Duration::from_millis(bd));

        let output = pomo.update();
        assert_eq!(pomo.task(), Some(&SimpleTask::new("Task2".into())));
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Break, PomoState::Working,))
        );

        // *************
        // complete work 2
        // *************
        std::thread::sleep(Duration::from_millis(wd));

        let output = pomo.update();
        let mut t1 = SimpleTask::new("Task2".into());
        t1.complete();
        assert_eq!(pomo.task(), Some(&SimpleTask::new("Task3".into())));
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new_task(
                PomoState::Working,
                PomoState::Break,
                t1
            ))
        );

        // *************
        // complete break 2
        // *************
        std::thread::sleep(Duration::from_millis(bd));

        let output = pomo.update();
        assert_eq!(pomo.task(), Some(&SimpleTask::new("Task3".into())));
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Break, PomoState::Working,))
        );

        // *************
        // complete work 3
        // *************
        std::thread::sleep(Duration::from_millis(wd));

        let output = pomo.update();
        assert_eq!(pomo.task(), None);
        // transition
        let mut t1 = SimpleTask::new("Task3".into());
        t1.complete();
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new_task(
                PomoState::Working,
                PomoState::Break,
                t1
            ))
        );

        // *************
        // complete break 3
        // *************
        std::thread::sleep(Duration::from_millis(bd));

        let output = pomo.update();
        assert_eq!(pomo.task(), None);
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Break, PomoState::Working,))
        );

        // *************
        // complete work 4
        // *************
        std::thread::sleep(Duration::from_millis(wd));

        let output = pomo.update();
        assert_eq!(pomo.task(), None);
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Working, PomoState::LongBreak,))
        );

        // *************
        // complete long break 1
        // *************
        std::thread::sleep(Duration::from_millis(rd));

        let output = pomo.update();
        assert_eq!(pomo.task(), None);
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::LongBreak, PomoState::Working,))
        );

        // *************
        // complete work 5
        // *************
        std::thread::sleep(Duration::from_millis(wd));

        let output = pomo.update();
        assert_eq!(pomo.task(), None);
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Working, PomoState::Break,))
        );

        // *************
        // complete break 4
        // *************
        std::thread::sleep(Duration::from_millis(bd));

        let output = pomo.update();
        assert_eq!(pomo.task(), None);
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Break, PomoState::Working,))
        );
        assert!(!pomo.is_completed());

        // *************
        // complete work 6
        // *************
        std::thread::sleep(Duration::from_millis(wd));

        let output = pomo.update();
        assert_eq!(pomo.task(), None);
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Working, PomoState::Completed,))
        );
        assert!(pomo.is_completed());
    }

    #[test]
    fn it_should_reset() {
        let mut pomo = SimplePomo::<SimpleTask, InstantTimer>::default();
        pomo.tasks.push(SimpleTask::new("Test"));
        assert!(!pomo.tasks.is_empty());
        pomo.clear();
        assert!(pomo.tasks.is_empty());
    }

    #[test]
    fn it_should_add_tasks() {
        let mut pomo = SimplePomo::<SimpleTask, InstantTimer>::default();
        assert_eq!(pomo.tasks.len(), 0);
        assert_eq!(
            pomo.execute(PomoCommand::AddTask(SimpleTask::new("Test"))),
            PomoMessage::Executed
        );
        assert_eq!(pomo.tasks.len(), 1);
    }

    #[test]
    fn it_should_remove_tasks() {
        let mut pomo = SimplePomo::<SimpleTask, InstantTimer>::default();
        assert_eq!(pomo.tasks.len(), 0);
        assert_eq!(
            pomo.execute(PomoCommand::AddTask(SimpleTask::new("Test1"))),
            PomoMessage::Executed
        );
        assert_eq!(
            pomo.execute(PomoCommand::AddTask(SimpleTask::new("Test2"))),
            PomoMessage::Executed
        );
        assert_eq!(
            pomo.execute(PomoCommand::AddTask(SimpleTask::new("Test3"))),
            PomoMessage::Executed
        );
        assert_eq!(pomo.tasks.len(), 3);
        assert_eq!(
            pomo.execute(PomoCommand::RemoveTask(1)),
            PomoMessage::Executed
        );
        assert_eq!(
            pomo.tasks,
            vec![SimpleTask::new("Test1"), SimpleTask::new("Test3")]
        );
    }
}
