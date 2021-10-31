use crate::{Promo, PromoState, Task};

pub struct PromoTransition {
    pub from: PromoState,
    pub to: PromoState,
}

impl PromoTransition {
    pub fn new(from: PromoState, to: PromoState) -> Self {
        Self { from, to }
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

pub enum PromoMessage<TTask>
where
    TTask: Task,
{
    Transition(PromoTransition),
    TaskCompleted(PromoTaskCompleted<TTask>),
}
