use std::marker::PhantomData;

use crate::{PomoState, Task};

#[derive(PartialEq, Eq, Debug)]
pub struct Transition<TTask, TError>
where
    TTask: Task<TError>,
{
    pub from: PomoState,
    pub to: PomoState,
    pub completed: Option<TTask>,
    phantom_error: PhantomData<TError>,
}

impl<TTask, TError> Transition<TTask, TError>
where
    TTask: Task<TError>,
{
    pub fn new(from: PomoState, to: PomoState) -> Self {
        Self {
            from,
            to,
            completed: None,
            phantom_error: PhantomData::default(),
        }
    }

    pub fn new_task(from: PomoState, to: PomoState, completed: TTask) -> Self {
        Self {
            from,
            to,
            completed: Some(completed),
            phantom_error: PhantomData::default(),
        }
    }
}
#[derive(PartialEq, Eq, Debug)]
pub enum PomoMessage<TTask, TError>
where
    TTask: Task<TError>,
{
    Transition(Transition<TTask, TError>),
    NoMessage,
}
