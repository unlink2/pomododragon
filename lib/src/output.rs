use std::marker::PhantomData;

use crate::{Promo, PromoState, Task, Timer};

pub struct PromoTransition {
    pub from: PromoState,
    pub to: PromoState,
}

impl PromoTransition {
    pub fn new(from: PromoState, to: PromoState) -> Self {
        Self { from, to }
    }
}

pub struct PromoUpdate<'a, TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    pub state: PromoState,
    pub timer: Option<&'a TTimer>,
    pub task: Option<&'a TTask>,
}

impl<'a, TTask, TTimer> PromoUpdate<'a, TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    pub fn new(state: PromoState, timer: Option<&'a TTimer>, task: Option<&'a TTask>) -> Self {
        Self { state, timer, task }
    }
}

pub struct PromoTaskCompleted<TTask>
where
    TTask: Task,
{
    pub task: TTask,
}

impl<TTask> PromoTaskCompleted<TTask>
where
    TTask: Task,
{
    pub fn new(task: TTask) -> Self {
        Self { task }
    }
}

pub enum PromoMessage<'a, TTask, TTimer>
where
    TTask: Task,
    TTimer: Timer,
{
    Transition(PromoTransition),
    Update(PromoUpdate<'a, TTask, TTimer>),
    TaskCompleted(PromoTaskCompleted<TTask>),
}

/// This is a receiver for output
/// This can be any system that can receive messages from
/// promo about its current state
pub trait OutputSystem<TTask, TTimer, TError>
where
    TTask: Task,
    TTimer: Timer,
{
    /// received every time the promo is updated
    fn update(
        &mut self,
        promo: &dyn Promo<TTask, TTimer, TError>,
        state: PromoState,
        timer: Option<&TTimer>,
        task: Option<&TTask>,
    ) -> Result<(), TError>;

    fn task_completed(
        &mut self,
        promo: &dyn Promo<TTask, TTimer, TError>,
        task: &TTask,
    ) -> Result<(), TError>;

    /// called for every state transition
    fn state_changed(
        &mut self,
        promo: &dyn Promo<TTask, TTimer, TError>,
        from: PromoState,
        to: PromoState,
    ) -> Result<(), TError>;
}

#[derive(Eq, PartialEq, Debug)]
pub enum SampleOutputCallee {
    Update,
    StateChanged,
    TaskCompelted,
}

#[derive(Debug, PartialEq)]
pub struct SampleOutputData<TTask>
where
    TTask: Task + PartialEq,
{
    pub callee: SampleOutputCallee,
    pub task: Option<TTask>,
    pub state: PromoState,
    pub from: Option<PromoState>,
}

// This is a sample output system
pub struct SampleOutputSystem<TTask, TTimer, TError>
where
    TTask: Task + Clone + PartialEq,
    TTimer: Timer + Clone,
{
    pub data: Vec<SampleOutputData<TTask>>, // contains a version of the output
    phantom_task: PhantomData<TTask>,
    phantom_timer: PhantomData<TTimer>,
    phantom_error: PhantomData<TError>,
}

impl<TTask, TTimer, TError> Default for SampleOutputSystem<TTask, TTimer, TError>
where
    TTask: Task + Clone + PartialEq,
    TTimer: Timer + Clone,
{
    fn default() -> Self {
        Self {
            data: vec![],
            phantom_task: PhantomData::default(),
            phantom_timer: PhantomData::default(),
            phantom_error: PhantomData::default(),
        }
    }
}

impl<TTask, TTimer, TError> OutputSystem<TTask, TTimer, TError>
    for SampleOutputSystem<TTask, TTimer, TError>
where
    TTask: Task + Clone + PartialEq,
    TTimer: Timer + Clone,
{
    /// received every time the promo is updated
    fn update(
        &mut self,
        _promo: &dyn Promo<TTask, TTimer, TError>,
        state: PromoState,
        _timer: Option<&TTimer>,
        task: Option<&TTask>,
    ) -> Result<(), TError> {
        self.data.push(SampleOutputData {
            callee: SampleOutputCallee::Update,
            task: if let Some(task) = task {
                Some(task.clone())
            } else {
                None
            },
            state,
            from: None,
        });

        Ok(())
    }

    fn task_completed(
        &mut self,
        promo: &dyn Promo<TTask, TTimer, TError>,
        task: &TTask,
    ) -> Result<(), TError> {
        self.data.push(SampleOutputData {
            callee: SampleOutputCallee::TaskCompelted,
            task: Some(task.clone()),
            state: promo.state(),
            from: None,
        });
        Ok(())
    }

    /// called for every state transition
    fn state_changed(
        &mut self,
        _promo: &dyn Promo<TTask, TTimer, TError>,
        from: PromoState,
        to: PromoState,
    ) -> Result<(), TError> {
        self.data.push(SampleOutputData {
            callee: SampleOutputCallee::Update,
            task: None,
            state: to,
            from: Some(from),
        });
        Ok(())
    }
}
