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
pub struct TaskCompleted<TTask>
where
    TTask: Task,
{
    pub task: TTask,
}

impl<TTask> TaskCompleted<TTask>
where
    TTask: Task,
{
    pub fn new(task: TTask) -> Self {
        Self { task }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum PromoMessage<TTask>
where
    TTask: Task,
{
    Transition(Transition),
    TaskCompleted(TaskCompleted<TTask>),
}
