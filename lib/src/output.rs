use crate::{Promo, PromoState, Task, Timer};

/// This is a receiver for output
/// This can be any system that can receive messages from
/// promo about its current state
pub trait OutputSystem<TTask, TTimer>: Default
where
    TTask: Task,
    TTimer: Timer,
{
    /// received every time the promo is updated
    fn update(
        &mut self,
        promo: &dyn Promo<TTask, TTimer, Self>,
        state: PromoState,
        timer: TTimer,
        task: Option<TTask>,
    );

    /// called for every state transition
    fn state_changed(
        &mut self,
        promo: &dyn Promo<TTask, TTimer, Self>,
        from: PromoState,
        to: PromoState,
    );
}
