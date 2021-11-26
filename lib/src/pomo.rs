use crate::{PomoMessage, Task, Timer, Transition};
use derive_builder::*;

/// Pomo is a simple state machine
/// with a timer and an output interface
pub trait Pomo<TTask, TTimer, TError>
where
    TTask: Task<TError>,
    TTimer: Timer<TError>,
{
    fn start(&mut self) -> Result<PomoMessage<TTask, TError>, TError>;

    fn reset(&mut self) -> Result<PomoMessage<TTask, TError>, TError>;

    /// shoudl call output.update
    /// and output.task_compelted if a task was completed!
    fn update(&mut self) -> Result<PomoMessage<TTask, TError>, TError>;

    fn state(&self) -> PomoState;

    /// returns the current task
    fn task(&self) -> Option<&TTask>;

    fn tasks(&self) -> &[TTask];
    fn tasks_mut(&mut self) -> &mut [TTask];

    /// returns the current timer
    fn timer(&self) -> Option<&TTimer>;

    /// Should call output.state_changed!
    fn set_state(&mut self, state: PomoState) -> Result<PomoMessage<TTask, TError>, TError>;

    fn toggle_pause(&mut self) -> Result<PomoMessage<TTask, TError>, TError>;
    fn pause(&mut self) -> Result<PomoMessage<TTask, TError>, TError>;
    fn unpause(&mut self) -> Result<PomoMessage<TTask, TError>, TError>;

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
    TTask: Task<()>,
    TTimer: Timer<()>,
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

    fn update_working(&mut self) -> Result<PomoMessage<TTask, ()>, ()> {
        // tick the timer
        let msg = if self.work_timer.is_completed() {
            self.current_cycles += 1;

            // remove first task and make it completed!
            let completed = if !self.tasks.is_empty() {
                let mut comp = self.tasks.remove(0);
                comp.complete()?;
                Some(comp)
            } else {
                None
            };

            // either long or regular break
            let mut msg = if self.current_cycles == self.total_cycles {
                // DONE!
                self.set_state(PomoState::Completed)?
            } else if self.current_cycles % self.cycles_until_long_break == 0 {
                self.long_break_timer.reset()?;
                self.set_state(PomoState::LongBreak)?
            } else {
                self.break_timer.reset()?;
                self.set_state(PomoState::Break)?
            };
            // if we did transition, set the completed task
            if let PomoMessage::Transition(transition) = &mut msg {
                transition.completed = completed;
            }
            msg
        } else {
            PomoMessage::NoMessage
        };
        Ok(msg)
    }

    fn update_break(&mut self) -> Result<PomoMessage<TTask, ()>, ()> {
        let msg = if self.break_timer.is_completed() {
            self.work_timer.reset()?;
            self.set_state(PomoState::Working)?
        } else {
            PomoMessage::NoMessage
        };
        Ok(msg)
    }

    fn update_long_break(&mut self) -> Result<PomoMessage<TTask, ()>, ()> {
        let msg = if self.long_break_timer.is_completed() {
            self.work_timer.start()?;
            self.set_state(PomoState::Working)?
        } else {
            PomoMessage::NoMessage
        };

        Ok(msg)
    }
}

impl<TTask, TTimer> Pomo<TTask, TTimer, ()> for SimplePomo<TTask, TTimer>
where
    TTask: Task<()>,
    TTimer: Timer<()>,
{
    fn start(&mut self) -> Result<PomoMessage<TTask, ()>, ()> {
        self.set_state(PomoState::Pending)
    }

    fn reset(&mut self) -> Result<PomoMessage<TTask, ()>, ()> {
        self.current_cycles = 0;
        self.state = PomoState::default();
        self.prev_state = PomoState::default();
        self.tasks.clear();

        Ok(PomoMessage::Reset)
    }

    fn update(&mut self) -> Result<PomoMessage<TTask, ()>, ()> {
        Ok(match self.state() {
            PomoState::NotStarted => PomoMessage::NoMessage,
            PomoState::Pending => {
                // start the timer and change state
                self.work_timer.reset()?;
                self.set_state(PomoState::Working)?
            }
            PomoState::Working => self.update_working()?,
            PomoState::Break => self.update_break()?,
            PomoState::LongBreak => self.update_long_break()?,
            PomoState::Paused | PomoState::Completed => PomoMessage::NoMessage,
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
            self.pause()
        } else {
            self.unpause()
        }
    }

    fn pause(&mut self) -> Result<PomoMessage<TTask, ()>, ()> {
        if !self.is_paused() {
            match self.state() {
                PomoState::Working => self.work_timer.pause(),
                PomoState::Break => self.break_timer.pause(),
                PomoState::LongBreak => self.long_break_timer.pause(),
                _ => (),
            }
            self.set_state(PomoState::Paused)
        } else {
            Ok(PomoMessage::Transition(Transition::new(
                PomoState::Paused,
                PomoState::Paused,
            )))
        }
    }

    fn unpause(&mut self) -> Result<PomoMessage<TTask, ()>, ()> {
        if self.is_paused() {
            match self.prev_state {
                PomoState::Working => self.work_timer.resume(),
                PomoState::Break => self.break_timer.resume(),
                PomoState::LongBreak => self.long_break_timer.resume(),
                _ => (),
            }
            self.set_state(self.prev_state)
        } else {
            Ok(PomoMessage::Transition(Transition::new(
                self.state, self.state,
            )))
        }
    }

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

        let output = pomo.start().unwrap();
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::NotStarted, PomoState::Pending,))
        );

        // not started
        let output = pomo.update().unwrap();

        // start working
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Pending, PomoState::Working,))
        );
        assert_eq!(pomo.task(), Some(&SimpleTask::new("Task1".into())));

        // *************
        // first update
        // *************
        let output = pomo.update().unwrap();
        assert_eq!(pomo.task(), Some(&SimpleTask::new("Task1".into())));
        assert_eq!(output, PomoMessage::NoMessage);

        // *************
        // complete first work
        // *************
        std::thread::sleep(Duration::from_millis(wd));
        let output = pomo.update().unwrap();
        let mut t1 = SimpleTask::new("Task1".into());
        // task completed call
        t1.complete().unwrap();
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
        let output = pomo.update().unwrap();
        assert_eq!(output, PomoMessage::NoMessage);

        // *************
        // attempt pause
        // *************
        let output = pomo.toggle_pause().unwrap();
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
        let output = pomo.toggle_pause().unwrap();
        // transition
        assert_eq!(
            output,
            PomoMessage::Transition(Transition::new(PomoState::Paused, PomoState::Break))
        );
        assert!(!pomo.break_timer.is_paused());
        assert!(!pomo.work_timer.is_paused());
        assert!(!pomo.long_break_timer.is_paused());

        let output = pomo.update().unwrap();
        assert_eq!(pomo.task(), Some(&SimpleTask::new("Task2".into())));
        assert_eq!(output, PomoMessage::NoMessage);

        // *************
        // complete break
        // *************
        std::thread::sleep(Duration::from_millis(bd));

        let output = pomo.update().unwrap();
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

        let output = pomo.update().unwrap();
        let mut t1 = SimpleTask::new("Task2".into());
        t1.complete().unwrap();
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

        let output = pomo.update().unwrap();
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

        let output = pomo.update().unwrap();
        assert_eq!(pomo.task(), None);
        // transition
        let mut t1 = SimpleTask::new("Task3".into());
        t1.complete().unwrap();
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

        let output = pomo.update().unwrap();
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

        let output = pomo.update().unwrap();
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

        let output = pomo.update().unwrap();
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

        let output = pomo.update().unwrap();
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

        let output = pomo.update().unwrap();
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

        let output = pomo.update().unwrap();
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
        pomo.reset().unwrap();
        assert!(pomo.tasks.is_empty());
    }
}
