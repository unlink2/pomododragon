use crate::{PomoState, Task};

#[derive(PartialEq, Eq, Debug)]
pub struct Transition<TTask>
where
    TTask: Task,
{
    pub from: PomoState,
    pub to: PomoState,
    pub completed: Option<TTask>,
}

impl<TTask> Transition<TTask>
where
    TTask: Task,
{
    pub fn new(from: PomoState, to: PomoState) -> Self {
        Self {
            from,
            to,
            completed: None,
        }
    }

    pub fn new_task(from: PomoState, to: PomoState, completed: TTask) -> Self {
        Self {
            from,
            to,
            completed: Some(completed),
        }
    }
}

impl<TTask> std::fmt::Display for Transition<TTask>
where
    TTask: Task,
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
pub enum PomoMessage<TTask>
where
    TTask: Task,
{
    Transition(Transition<TTask>),
    NoMessage,
    Executed,
    Reset,
}

impl<TTask> std::fmt::Display for PomoMessage<TTask>
where
    TTask: Task,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Transition(t) => t.to_string(),
                Self::NoMessage => "NoMessage".into(),
                Self::Reset => "Reset".into(),
                Self::Executed => "Executed".into(),
            }
        )
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PomoCommand<TTask>
where
    TTask: Task,
{
    AddTask(TTask),
    RemoveTask(usize),
    Start,
    Reset,
    Pause,
    Unpause,
    TogglePause,
    Update,
    Clear,
}
