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

impl<TTask, TError> std::fmt::Display for Transition<TTask, TError>
where
    TTask: Task<TError>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "(from: {}, to: {}, completed: {})",
            self.from,
            self.to,
            match &self.completed {
                Some(task) => task.to_string(),
                None => "None".into(),
            }
        )
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum PomoMessage<TTask, TError>
where
    TTask: Task<TError>,
{
    Transition(Transition<TTask, TError>),
    NoMessage,
    Reset,
}

impl<TTask, TError> std::fmt::Display for PomoMessage<TTask, TError>
where
    TTask: Task<TError>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Transition(t) => t.to_string(),
                Self::NoMessage => "NoMessage".into(),
                Self::Reset => "Reset".into(),
            }
        )
    }
}
