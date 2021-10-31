use std::marker::PhantomData;

use crate::{OutputSystem, Task, Timer};
use derive_builder::*;

/// Promo is a simple state machine
/// with a timer and an output interface
pub trait Promo<TTask, TTimer, TError>
where
    TTask: Task,
    TTimer: Timer,
{
    /// shoudl call output.update
    /// and output.task_compelted if a task was completed!
    fn update(
        &mut self,
        output: &mut dyn OutputSystem<TTask, TTimer, TError>,
    ) -> Result<(), TError>;

    fn state(&self) -> PromoState;

    /// Should call output.state_changed!
    fn set_state(
        &mut self,
        state: PromoState,
        output: &mut dyn OutputSystem<TTask, TTimer, TError>,
    ) -> Result<(), TError>;

    fn toggle_pause(
        &mut self,
        output: &mut dyn OutputSystem<TTask, TTimer, TError>,
    ) -> Result<bool, TError>;

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
pub struct SimplePromo<TTask, TTimer, TError>
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

    #[builder(default = "PhantomData::default()")]
    phantom_error: PhantomData<TError>,
}

impl<TTask, TTimer, TError> Default for SimplePromo<TTask, TTimer, TError>
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

impl<TTask, TTimer, TError> SimplePromo<TTask, TTimer, TError>
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
            phantom_error: PhantomData::default(),
        }
    }

    fn update_working(
        &mut self,
        output: &mut dyn OutputSystem<TTask, TTimer, TError>,
    ) -> Result<(), TError> {
        output.update(
            self,
            self.state(),
            Some(&self.work_timer),
            self.tasks.get(0),
        )?;
        // tick the timer
        if self.work_timer.is_completed() {
            self.current_cycles += 1;

            // remove first task and make it completed!
            if !self.tasks.is_empty() {
                let comp = self.tasks.remove(0);
                output.task_completed(self, &comp)?;
            }

            // either rest or break
            if self.current_cycles == self.total_cycles {
                // DONE!
                self.set_state(PromoState::Completed, output)?;
            } else if self.current_cycles % self.cycles_until_rest == 0 {
                self.set_state(PromoState::Resting, output)?;
                self.rest_timer.reset();
            } else {
                self.set_state(PromoState::Break, output)?;
                self.break_timer.reset();
            }
        }

        Ok(())
    }

    fn update_break(
        &mut self,
        output: &mut dyn OutputSystem<TTask, TTimer, TError>,
    ) -> Result<(), TError> {
        output.update(
            self,
            self.state(),
            Some(&self.break_timer),
            self.tasks.get(0),
        )?;

        if self.break_timer.is_completed() {
            self.set_state(PromoState::Working, output)?;
            self.work_timer.reset();
        }

        Ok(())
    }

    fn update_resting(
        &mut self,
        output: &mut dyn OutputSystem<TTask, TTimer, TError>,
    ) -> Result<(), TError> {
        output.update(
            self,
            self.state(),
            Some(&self.break_timer),
            self.tasks.get(0),
        )?;

        if self.rest_timer.is_completed() {
            self.set_state(PromoState::Working, output)?;
            self.work_timer.start();
        }
        Ok(())
    }
}

impl<TTask, TTimer, TError> Promo<TTask, TTimer, TError> for SimplePromo<TTask, TTimer, TError>
where
    TTask: Task,
    TTimer: Timer,
{
    fn update(
        &mut self,
        output: &mut dyn OutputSystem<TTask, TTimer, TError>,
    ) -> Result<(), TError> {
        match self.state() {
            PromoState::NotStarted => {
                // start the timer and change state
                self.work_timer.reset();
                self.set_state(PromoState::Working, output)?;
            }
            PromoState::Working => self.update_working(output)?,
            PromoState::Break => self.update_break(output)?,
            PromoState::Resting => self.update_resting(output)?,
            PromoState::Paused | PromoState::Completed => {
                output.update(self, self.state(), None, None)?
            }
        }

        Ok(())
    }

    /// Should call output.state_changed!
    fn set_state(
        &mut self,
        state: PromoState,
        output: &mut dyn OutputSystem<TTask, TTimer, TError>,
    ) -> Result<(), TError> {
        self.prev_state = self.state();
        self.state = state;
        output.state_changed(self, self.prev_state, self.state())?;
        Ok(())
    }

    fn toggle_pause(
        &mut self,
        output: &mut dyn OutputSystem<TTask, TTimer, TError>,
    ) -> Result<bool, TError> {
        if !self.is_paused() {
            match self.state {
                PromoState::Working => self.work_timer.pause(),
                PromoState::Break => self.break_timer.pause(),
                PromoState::Resting => self.rest_timer.pause(),
                _ => (),
            }
            self.set_state(PromoState::Paused, output)?;
        } else {
            self.set_state(self.prev_state, output)?;
            match self.state {
                PromoState::Working => self.work_timer.resume(),
                PromoState::Break => self.break_timer.resume(),
                PromoState::Resting => self.rest_timer.resume(),
                _ => (),
            }
        }

        Ok(self.is_paused())
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

    use crate::{InstantTimer, SampleOutputData, SampleOutputSystem, SimpleTask};

    use super::*;

    #[test]
    fn it_should_update_states_in_order() {
        let mut promo = SimplePromoBuilder::<SimpleTask, InstantTimer, ()>::default()
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
        let mut output = SampleOutputSystem::default();

        // not started
        assert!(output.data.is_empty());

        promo.update(&mut output).unwrap();

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
