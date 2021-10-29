use std::marker::PhantomData;

use crate::{Promo, PromoState, Task, Timer};

/// This is a receiver for output
/// This can be any system that can receive messages from
/// promo about its current state
pub trait OutputSystem<TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    /// received every time the promo is updated
    fn update(
        &mut self,
        promo: &dyn Promo<TTask, TTimer>,
        state: PromoState,
        timer: Option<&TTimer>,
        task: Option<&TTask>,
    );

    fn task_completed(&mut self, promo: &dyn Promo<TTask, TTimer>, task: &TTask);

    /// called for every state transition
    fn state_changed(&mut self, promo: &dyn Promo<TTask, TTimer>, from: PromoState, to: PromoState);
}

pub struct SampleOutputData<TTask>
where
    TTask: Task + PartialEq,
{
    task: Option<TTask>,
    state: PromoState,
    from: Option<PromoState>,
}

// This is a sample output system
pub struct SampleOutputSystem<TTask, TTimer>
where
    TTask: Task + Clone + PartialEq,
    TTimer: Timer + Clone,
{
    pub data: Vec<SampleOutputData<TTask>>, // contains a version of the output
    phantom_task: PhantomData<TTask>,
    phantom_timer: PhantomData<TTimer>,
}

impl<TTask, TTimer> SampleOutputSystem<TTask, TTimer>
where
    TTask: Task + Clone + PartialEq,
    TTimer: Timer + Clone,
{
}

impl<TTask, TTimer> OutputSystem<TTask, TTimer> for SampleOutputSystem<TTask, TTimer>
where
    TTask: Task + Clone + PartialEq,
    TTimer: Timer + Clone,
{
    /// received every time the promo is updated
    fn update(
        &mut self,
        _promo: &dyn Promo<TTask, TTimer>,
        state: PromoState,
        _timer: Option<&TTimer>,
        task: Option<&TTask>,
    ) {
        self.data.push(SampleOutputData {
            task: if let Some(task) = task {
                Some(task.clone())
            } else {
                None
            },
            state,
            from: None,
        });
    }

    fn task_completed(&mut self, promo: &dyn Promo<TTask, TTimer>, task: &TTask) {
        self.data.push(SampleOutputData {
            task: Some(task.clone()),
            state: promo.state(),
            from: None,
        });
    }

    /// called for every state transition
    fn state_changed(
        &mut self,
        _promo: &dyn Promo<TTask, TTimer>,
        from: PromoState,
        to: PromoState,
    ) {
        self.data.push(SampleOutputData {
            task: None,
            state: to,
            from: Some(from),
        });
    }
}
