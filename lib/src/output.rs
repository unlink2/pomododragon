use std::marker::PhantomData;

use crate::{PromoState, Task};

#[derive(PartialEq, Eq, Debug)]
pub struct Transition {
    pub from: PromoState,
    pub to: PromoState,
}

impl Transition {
    pub fn new(from: PromoState, to: PromoState) -> Self {
        Self { from, to }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct TaskCompleted<TTask, TError>
where
    TTask: Task<TError>,
{
    pub task: TTask,
    phantom_error: PhantomData<TError>,
}

impl<TTask, TError> TaskCompleted<TTask, TError>
where
    TTask: Task<TError>,
{
    pub fn new(task: TTask) -> Self {
        Self {
            task,
            phantom_error: PhantomData::default(),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum PromoMessage<TTask, TError>
where
    TTask: Task<TError>,
{
    Transition(Transition),
    TaskCompleted(TaskCompleted<TTask, TError>),
}
