use crate::{PomoMessage, Task, TaskCompleted, Timer, Transition};
use derive_builder::*;

/// Pomo is a simple state machine
/// with a timer and an output interface
pub trait Pomo<TTask, TTimer, TError>
where
    TTask: Task<TError>,
    TTimer: Timer<TError>,
{
    /// shoudl call output.update
    /// and output.task_compelted if a task was completed!
    fn update(&mut self) -> Result<Vec<PomoMessage<TTask, TError>>, TError>;

    fn state(&self) -> PomoState;

    /// returns the current task
    fn task(&self) -> Option<&TTask>;

    /// returns the current timer
    fn timer(&self) -> Option<&TTimer>;

    /// Should call output.state_changed!
    fn set_state(&mut self, state: PomoState) -> Result<PomoMessage<TTask, TError>, TError>;

    fn toggle_pause(&mut self) -> Result<PomoMessage<TTask, TError>, TError>;

    fn is_paused(&self) -> bool {
        self.state() == PomoState::Paused
    }

    fn is_completed(&self) -> bool {
        self.state() == PomoState::Completed
    }
}

/// Current state
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PomoState {
    NotStarted,
    Working,
    Break,
    LongBreak,
    Completed,
    Paused,
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
    TTask: Task<()>,
    TTimer: Timer<()>,
{
    #[builder(default)]
    tasks: Vec<TTask>,
    #[builder(default = "TTimer::default_break_timer()")]
    break_timer: TTimer, // short break
    #[builder(default = "TTimer::default_work_timer()")]
    work_timer: TTimer, // work
    #[builder(default = "TTimer::default_long_break_timer()")]
    long_break_timer: TTimer, // long break,

    // cycle counts
    #[builder(default = "0")]
    current_cycles: usize,
    #[builder(default = "4")]
    cycles_until_long_break: usize,
    #[builder(default = "8")]
    total_cycles: usize,

    #[builder(default = "PomoState::default()")]
    state: PomoState, // internal state
    #[builder(default = "PomoState::default()")]
    prev_state: PomoState,
}

impl<TTask, TTimer> Default for SimplePomo<TTask, TTimer>
where
    TTask: Task<()>,
    TTimer: Timer<()>,
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
    TTask: Task<()>,
    TTimer: Timer<()>,
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

    fn update_working(&mut self) -> Result<Vec<PomoMessage<TTask, ()>>, ()> {
        let mut msgs = vec![];
        // tick the timer
        if self.work_timer.is_completed() {
            self.current_cycles += 1;

            // remove first task and make it completed!
            if !self.tasks.is_empty() {
                let mut comp = self.tasks.remove(0);
                comp.complete()?;
                msgs.push(PomoMessage::TaskCompleted(TaskCompleted::new(comp)));
            }

            // either long or regular break
            if self.current_cycles == self.total_cycles {
                // DONE!
                msgs.push(self.set_state(PomoState::Completed)?);
            } else if self.current_cycles % self.cycles_until_long_break == 0 {
                msgs.push(self.set_state(PomoState::LongBreak)?);
                self.long_break_timer.reset()?;
            } else {
                msgs.push(self.set_state(PomoState::Break)?);
                self.break_timer.reset()?;
            }
        }
        Ok(msgs)
    }

    fn update_break(&mut self) -> Result<Vec<PomoMessage<TTask, ()>>, ()> {
        let mut msgs = vec![];

        if self.break_timer.is_completed() {
            msgs.push(self.set_state(PomoState::Working)?);
            self.work_timer.reset()?;
        }
        Ok(msgs)
    }

    fn update_long_break(&mut self) -> Result<Vec<PomoMessage<TTask, ()>>, ()> {
        let mut msgs = vec![];

        if self.long_break_timer.is_completed() {
            msgs.push(self.set_state(PomoState::Working)?);
            self.work_timer.start()?;
        }

        Ok(msgs)
    }
}

impl<TTask, TTimer> Pomo<TTask, TTimer, ()> for SimplePomo<TTask, TTimer>
where
    TTask: Task<()>,
    TTimer: Timer<()>,
{
    fn update(&mut self) -> Result<Vec<PomoMessage<TTask, ()>>, ()> {
        Ok(match self.state() {
            PomoState::NotStarted => {
                // start the timer and change state
                self.work_timer.reset()?;
                vec![self.set_state(PomoState::Working)?]
            }
            PomoState::Working => self.update_working()?,
            PomoState::Break => self.update_break()?,
            PomoState::LongBreak => self.update_long_break()?,
            PomoState::Paused | PomoState::Completed => vec![],
        })
    }

    /// Should call output.state_changed!
    fn set_state(&mut self, state: PomoState) -> Result<PomoMessage<TTask, ()>, ()> {
        self.prev_state = self.state();
        self.state = state;
        Ok(PomoMessage::Transition(Transition::new(
            self.prev_state,
            self.state(),
        )))
    }

    fn toggle_pause(&mut self) -> Result<PomoMessage<TTask, ()>, ()> {
        if !self.is_paused() {
            match self.state() {
                PomoState::Working => self.work_timer.pause(),
                PomoState::Break => self.break_timer.pause(),
                PomoState::LongBreak => self.long_break_timer.pause(),
                _ => (),
            }
            self.set_state(PomoState::Paused)
        } else {
            match self.prev_state {
                PomoState::Working => self.work_timer.resume(),
                PomoState::Break => self.break_timer.resume(),
                PomoState::LongBreak => self.long_break_timer.resume(),
                _ => (),
            }
            self.set_state(self.prev_state)
        }
    }

    fn task(&self) -> Option<&TTask> {
        self.tasks.get(0)
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

        let mut promo = SimplePomoBuilder::<SimpleTask, InstantTimer>::default()
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

        // not started
        let mut output = promo.update().unwrap();

        // start working
        assert_eq!(
            output.pop(),
            Some(PomoMessage::Transition(Transition::new(
                PomoState::NotStarted,
                PomoState::Working,
            )))
        );
        assert_eq!(promo.task(), Some(&SimpleTask::new("Task1".into())));
        assert!(output.is_empty());

        // *************
        // first update
        // *************
        let output = promo.update().unwrap();
        assert_eq!(promo.task(), Some(&SimpleTask::new("Task1".into())));
        assert!(output.is_empty());

        // *************
        // complete first work
        // *************
        std::thread::sleep(Duration::from_millis(wd));
        let mut output = promo.update().unwrap();
        // transition
        assert_eq!(
            output.pop(),
            Some(PomoMessage::Transition(Transition::new(
                PomoState::Working,
                PomoState::Break,
            )))
        );
        // task completed call
        let mut t1 = SimpleTask::new("Task1".into());
        t1.complete().unwrap();
        assert_eq!(
            output.pop(),
            Some(PomoMessage::TaskCompleted(TaskCompleted::new(t1)))
        );

        assert!(output.is_empty());

        // *************
        // update on break
        // *************
        let output = promo.update().unwrap();
        assert!(output.is_empty());

        // *************
        // attempt pause
        // *************
        let output = promo.toggle_pause().unwrap();
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Break, PomoState::Paused))
        );

        assert!(promo.break_timer.is_paused());
        assert!(!promo.work_timer.is_paused());
        assert!(!promo.long_break_timer.is_paused());

        // should still be paused!
        std::thread::sleep(Duration::from_millis(pd));
        let output = promo.toggle_pause().unwrap();
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Paused, PomoState::Break))
        );
        assert!(!promo.break_timer.is_paused());
        assert!(!promo.work_timer.is_paused());
        assert!(!promo.long_break_timer.is_paused());

        let output = promo.update().unwrap();
        assert_eq!(promo.task(), Some(&SimpleTask::new("Task2".into())));
        assert!(output.is_empty());

        // *************
        // complete break
        // *************
        std::thread::sleep(Duration::from_millis(bd));

        let mut output = promo.update().unwrap();
        assert_eq!(promo.task(), Some(&SimpleTask::new("Task2".into())));
        // transition
        assert_eq!(
            output.pop(),
            Some(PomoMessage::Transition(Transition::new(
                PomoState::Break,
                PomoState::Working,
            )))
        );
        assert!(output.is_empty());

        // *************
        // complete work 2
        // *************
        std::thread::sleep(Duration::from_millis(wd));

        let mut output = promo.update().unwrap();
        assert_eq!(promo.task(), Some(&SimpleTask::new("Task3".into())));
        // transition
        assert_eq!(
            output.pop(),
            Some(PomoMessage::Transition(Transition::new(
                PomoState::Working,
                PomoState::Break,
            )))
        );
        let mut t1 = SimpleTask::new("Task2".into());
        t1.complete().unwrap();
        assert_eq!(
            output.pop(),
            Some(PomoMessage::TaskCompleted(TaskCompleted::new(t1)))
        );
        assert!(output.is_empty());

        // *************
        // complete break 2
        // *************
        std::thread::sleep(Duration::from_millis(bd));

        let mut output = promo.update().unwrap();
        assert_eq!(promo.task(), Some(&SimpleTask::new("Task3".into())));
        // transition
        assert_eq!(
            output.pop(),
            Some(PomoMessage::Transition(Transition::new(
                PomoState::Break,
                PomoState::Working,
            )))
        );
        assert!(output.is_empty());

        // *************
        // complete work 3
        // *************
        std::thread::sleep(Duration::from_millis(wd));

        let mut output = promo.update().unwrap();
        assert_eq!(promo.task(), None);
        // transition
        assert_eq!(
            output.pop(),
            Some(PomoMessage::Transition(Transition::new(
                PomoState::Working,
                PomoState::Break,
            )))
        );
        let mut t1 = SimpleTask::new("Task3".into());
        t1.complete().unwrap();
        assert_eq!(
            output.pop(),
            Some(PomoMessage::TaskCompleted(TaskCompleted::new(t1)))
        );
        assert!(output.is_empty());

        // *************
        // complete break 3
        // *************
        std::thread::sleep(Duration::from_millis(bd));

        let mut output = promo.update().unwrap();
        assert_eq!(promo.task(), None);
        // transition
        assert_eq!(
            output.pop(),
            Some(PomoMessage::Transition(Transition::new(
                PomoState::Break,
                PomoState::Working,
            )))
        );
        assert!(output.is_empty());

        // *************
        // complete work 4
        // *************
        std::thread::sleep(Duration::from_millis(wd));

        let mut output = promo.update().unwrap();
        assert_eq!(promo.task(), None);
        // transition
        assert_eq!(
            output.pop(),
            Some(PomoMessage::Transition(Transition::new(
                PomoState::Working,
                PomoState::LongBreak,
            )))
        );
        assert!(output.is_empty());

        // *************
        // complete long break 1
        // *************
        std::thread::sleep(Duration::from_millis(rd));

        let mut output = promo.update().unwrap();
        assert_eq!(promo.task(), None);
        // transition
        assert_eq!(
            output.pop(),
            Some(PomoMessage::Transition(Transition::new(
                PomoState::LongBreak,
                PomoState::Working,
            )))
        );
        assert!(output.is_empty());

        // *************
        // complete work 5
        // *************
        std::thread::sleep(Duration::from_millis(wd));

        let mut output = promo.update().unwrap();
        assert_eq!(promo.task(), None);
        // transition
        assert_eq!(
            output.pop(),
            Some(PomoMessage::Transition(Transition::new(
                PomoState::Working,
                PomoState::Break,
            )))
        );
        assert!(output.is_empty());

        // *************
        // complete break 4
        // *************
        std::thread::sleep(Duration::from_millis(bd));

        let mut output = promo.update().unwrap();
        assert_eq!(promo.task(), None);
        // transition
        assert_eq!(
            output.pop(),
            Some(PomoMessage::Transition(Transition::new(
                PomoState::Break,
                PomoState::Working,
            )))
        );
        assert!(output.is_empty());
        assert!(!promo.is_completed());

        // *************
        // complete work 6
        // *************
        std::thread::sleep(Duration::from_millis(wd));

        let mut output = promo.update().unwrap();
        assert_eq!(promo.task(), None);
        // transition
        assert_eq!(
            output.pop(),
            Some(PomoMessage::Transition(Transition::new(
                PomoState::Working,
                PomoState::Completed,
            )))
        );
        assert!(output.is_empty());
        assert!(promo.is_completed());
    }
}
