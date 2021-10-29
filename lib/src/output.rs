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
