use std::marker::PhantomData;

use crate::{PomoState, Task};

#[derive(PartialEq, Eq, Debug)]
pub struct Transition {
    pub from: PomoState,
    pub to: PomoState,
}

impl Transition {
    pub fn new(from: PomoState, to: PomoState) -> Self {
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
pub enum PomoMessage<TTask, TError>
where
    TTask: Task<TError>,
{
    Transition(Transition),
    TaskCompleted(TaskCompleted<TTask, TError>),
}
