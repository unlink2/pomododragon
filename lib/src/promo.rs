use crate::{PromoMessage, PromoTaskCompleted, PromoTransition, PromoUpdate, Task, Timer};
use derive_builder::*;

/// Promo is a simple state machine
/// with a timer and an output interface
pub trait Promo<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    /// shoudl call output.update
    /// and output.task_compelted if a task was completed!
    fn update(&mut self) -> Vec<PromoMessage<TTask>>;

    fn state(&self) -> PromoState;

    /// returns the current task
    fn task(&self) -> Option<&TTask>;

    /// returns the current timer
    fn timer(&self) -> Option<&TTimer>;

    /// Should call output.state_changed!
    fn set_state(&mut self, state: PromoState) -> PromoMessage<TTask>;

    fn toggle_pause(&mut self) -> PromoMessage<TTask>;

    fn is_paused(&self) -> bool {
        self.state() == PromoState::Paused
    }

    fn is_completed(&self) -> bool {
        self.state() == PromoState::Completed
    }
}

/// Current state
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PromoState {
    NotStarted,
    Working,
    Break,
    Resting,
    Completed,
    Paused,
}

impl Default for PromoState {
    fn default() -> Self {
        Self::NotStarted
    }
}

/// A simple state machine
/// with a timer
#[derive(Builder, Debug)]
#[builder(setter(into))]
pub struct SimplePromo<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    #[builder(default)]
    tasks: Vec<TTask>,
    #[builder(default = "TTimer::default_break_timer()")]
    break_timer: TTimer, // short break
    #[builder(default = "TTimer::default_work_timer()")]
    work_timer: TTimer, // work
    #[builder(default = "TTimer::default_rest_timer()")]
    rest_timer: TTimer, // long break,

    // cycle counts
    #[builder(default = "0")]
    current_cycles: usize,
    #[builder(default = "4")]
    cycles_until_rest: usize,
    #[builder(default = "8")]
    total_cycles: usize,

    #[builder(default = "PromoState::default()")]
    state: PromoState, // internal state
    #[builder(default = "PromoState::default()")]
    prev_state: PromoState,
}

impl<TTask, TTimer> Default for SimplePromo<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    fn default() -> Self {
        Self::new(
            vec![],
            TTimer::default_work_timer(),
            TTimer::default_break_timer(),
            TTimer::default_rest_timer(),
        )
    }
}

impl<TTask, TTimer> SimplePromo<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    pub fn new(
        tasks: Vec<TTask>,
        work_timer: TTimer,
        break_timer: TTimer,
        rest_timer: TTimer,
    ) -> Self {
        Self {
            tasks,
            work_timer,
            break_timer,
            rest_timer,
            total_cycles: 8,
            cycles_until_rest: 4,
            current_cycles: 0,
            state: PromoState::default(),
            prev_state: PromoState::default(),
        }
    }

    fn update_working(&mut self) -> Vec<PromoMessage<TTask>> {
        let mut msgs = vec![PromoMessage::Update(PromoUpdate::new(
            self.state(),
            Some(&self.work_timer),
            self.tasks.get(0),
        ))];
        // tick the timer
        if self.work_timer.is_completed() {
            self.current_cycles += 1;

            // remove first task and make it completed!
            if !self.tasks.is_empty() {
                let comp = self.tasks.remove(0);
                msgs.push(PromoMessage::TaskCompleted(PromoTaskCompleted::new(comp)));
            }

            // either rest or break
            if self.current_cycles == self.total_cycles {
                // DONE!
                msgs.push(self.set_state(PromoState::Completed));
            } else if self.current_cycles % self.cycles_until_rest == 0 {
                msgs.push(self.set_state(PromoState::Resting));
                self.rest_timer.reset();
            } else {
                msgs.push(self.set_state(PromoState::Break));
                self.break_timer.reset();
            }
        }
        msgs
    }

    fn update_break(&mut self) -> Vec<PromoMessage<TTask>> {
        let mut msgs = vec![PromoMessage::Update(PromoUpdate::new(
            self.state(),
            Some(&self.work_timer),
            self.tasks.get(0),
        ))];

        if self.break_timer.is_completed() {
            msgs.push(self.set_state(PromoState::Working));
            self.work_timer.reset();
        }
        msgs
    }

    fn update_resting(&mut self) -> Vec<PromoMessage<TTask>> {
        let mut msgs = vec![PromoMessage::Update(PromoUpdate::new(
            self.state(),
            Some(&self.work_timer),
            self.tasks.get(0),
        ))];

        if self.rest_timer.is_completed() {
            msgs.push(self.set_state(PromoState::Working));
            self.work_timer.start();
        }

        msgs
    }
}

impl<TTask, TTimer> Promo<TTask, TTimer> for SimplePromo<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    fn update(&mut self) -> Vec<PromoMessage<TTask>> {
        match self.state() {
            PromoState::NotStarted => {
                // start the timer and change state
                self.work_timer.reset();
                vec![self.set_state(PromoState::Working)]
            }
            PromoState::Working => self.update_working(),
            PromoState::Break => self.update_break(),
            PromoState::Resting => self.update_resting(),
            PromoState::Paused | PromoState::Completed => {
                vec![PromoMessage::Update(PromoUpdate::new(
                    self.state(),
                    None,
                    None,
                ))]
            }
        }
    }

    /// Should call output.state_changed!
    fn set_state(&mut self, state: PromoState) -> PromoMessage<TTask> {
        self.prev_state = self.state();
        self.state = state;
        PromoMessage::Transition(PromoTransition::new(self.prev_state, self.state()))
    }

    fn toggle_pause(&mut self) -> PromoMessage<TTask> {
        if !self.is_paused() {
            match self.state() {
                PromoState::Working => self.work_timer.pause(),
                PromoState::Break => self.break_timer.pause(),
                PromoState::Resting => self.rest_timer.pause(),
                _ => (),
            }
            self.set_state(PromoState::Paused)
        } else {
            match self.prev_state {
                PromoState::Working => self.work_timer.resume(),
                PromoState::Break => self.break_timer.resume(),
                PromoState::Resting => self.rest_timer.resume(),
                _ => (),
            }
            self.set_state(self.prev_state)
        }
    }

    fn is_paused(&self) -> bool {
        self.state() == PromoState::Paused
    }

    fn state(&self) -> PromoState {
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
        let mut promo = SimplePromoBuilder::<SimpleTask, InstantTimer>::default()
            .break_timer(InstantTimer::new(Duration::from_millis(100)))
            .work_timer(InstantTimer::new(Duration::from_millis(200)))
            .rest_timer(InstantTimer::new(Duration::from_millis(250)))
            .tasks(vec![
                SimpleTask::new("Task1".into()),
                SimpleTask::new("Task2".into()),
                SimpleTask::new("Task3".into()),
            ])
            .build()
            .unwrap();

        // not started
        let output = promo.update();

        // start working
        assert_eq!(
            output.data.pop(),
            Some(SampleOutputData {
                task: None,
                state: PromoState::Working,
                from: Some(PromoState::NotStarted)
            })
        );
        assert!(output.data.is_empty());

        // *************
        // first update
        // *************
        promo.update(&mut output).unwrap();
        assert_eq!(
            output.data.pop(),
            Some(SampleOutputData {
                task: Some(SimpleTask::new("Task1".into())),
                state: PromoState::Working,
                from: None
            })
        );
        assert!(output.data.is_empty());

        // *************
        // complete first work
        // *************
        std::thread::sleep(Duration::from_millis(201));
        promo.update(&mut output).unwrap();
        // transition
        assert_eq!(
            output.data.pop(),
            Some(SampleOutputData {
                task: None,
                state: PromoState::Break,
                from: Some(PromoState::Working),
            })
        );
        // update call
        assert_eq!(
            output.data.pop(),
            Some(SampleOutputData {
                task: Some(SimpleTask::new("Task1".into())),
                state: PromoState::Working,
                from: None
            })
        );
        // task completed call
        assert_eq!(
            output.data.pop(),
            Some(SampleOutputData {
                task: Some(SimpleTask::new("Task1".into())),
                state: PromoState::Working,
                from: None
            })
        );
        assert!(output.data.is_empty());

        // *************
        // update on break
        // *************
        promo.update(&mut output).unwrap();
        assert_eq!(
            output.data.pop(),
            Some(SampleOutputData {
                task: Some(SimpleTask::new("Task2".into())),
                state: PromoState::Break,
                from: None
            })
        );
        assert!(output.data.is_empty());

        // *************
        // attempt pause
        // *************
        promo.toggle_pause(&mut output).unwrap();
        // transition
        assert_eq!(
            output.data.pop(),
            Some(SampleOutputData {
                task: None,
                state: PromoState::Paused,
                from: Some(PromoState::Break),
            })
        );
        assert!(output.data.is_empty());
        assert!(promo.break_timer.is_paused());
        assert!(!promo.work_timer.is_paused());
        assert!(!promo.rest_timer.is_paused());

        // should still be paused!
        std::thread::sleep(Duration::from_millis(500));
        promo.toggle_pause(&mut output).unwrap();
        // transition
        assert_eq!(
            output.data.pop(),
            Some(SampleOutputData {
                task: None,
                state: PromoState::Break,
                from: Some(PromoState::Paused),
            })
        );
        assert!(output.data.is_empty());
        assert!(!promo.break_timer.is_paused());
        assert!(!promo.work_timer.is_paused());
        assert!(!promo.rest_timer.is_paused());

        promo.update(&mut output).unwrap();
        assert_eq!(
            output.data.pop(),
            Some(SampleOutputData {
                task: Some(SimpleTask::new("Task2".into())),
                state: PromoState::Break,
                from: None
            })
        );
        assert!(output.data.is_empty());

        // *************
        // complete break
        // *************
        std::thread::sleep(Duration::from_millis(101));

        promo.update(&mut output).unwrap();
        assert_eq!(
            output.data.pop(),
            Some(SampleOutputData {
                task: None,
                state: PromoState::Break,
                from: None
            })
        );
        assert_eq!(
            output.data.pop(),
            Some(SampleOutputData {
                task: Some(SimpleTask::new("Task2".into())),
                state: PromoState::Break,
                from: None
            })
        );
        assert!(output.data.is_empty());
    }
}
