use crate::{OutputSystem, Task, Timer};
use derive_builder::*;

pub trait Promo<TTask, TTimer, TOutput>
where
    TTask: Task,
    TTimer: Timer,
    TOutput: OutputSystem<TTask, TTimer>,
{
}

/// Current state
#[derive(Copy, Clone, Debug)]
pub enum PromoState {
    NotStarted,
    Working,
    Break,
    Resting,
    Finished,
}

impl Default for PromoState {
    fn default() -> Self {
        Self::NotStarted
    }
}

/// A simple state machine
/// with a timer
#[derive(Builder, Debug)]
#[builder(setter(into))]
pub struct BasicPromo<TTask, TTimer, TOutput>
where
    TTask: Task,
    TTimer: Timer,
    TOutput: OutputSystem<TTask, TTimer>,
{
    tasks: Vec<TTask>,
    break_timer: TTimer, // short break
    work_timer: TTimer,  // work
    rest_timer: TTimer,  // long break,
    output: TOutput,

    // cycle counts
    current_cycles: usize,
    cycles_until_rest: usize,
    total_cycles: usize,
    state: PromoState, // internal state
}

impl<TTask, TTimer, TOutput> Default for BasicPromo<TTask, TTimer, TOutput>
where
    TTask: Task,
    TTimer: Timer,
    TOutput: OutputSystem<TTask, TTimer>,
{
    fn default() -> Self {
        Self::new(
            vec![],
            TTimer::default_work_timer(),
            TTimer::default_break_timer(),
            TTimer::default_rest_timer(),
            TOutput::default(),
        )
    }
}

impl<TTask, TTimer, TOutput> BasicPromo<TTask, TTimer, TOutput>
where
    TTask: Task,
    TTimer: Timer,
    TOutput: OutputSystem<TTask, TTimer>,
{
    pub fn new(
        tasks: Vec<TTask>,
        work_timer: TTimer,
        break_timer: TTimer,
        rest_timer: TTimer,
        output: TOutput,
    ) -> Self {
        Self {
            tasks,
            work_timer,
            break_timer,
            rest_timer,
            output,
            total_cycles: 8,
            cycles_until_rest: 4,
            current_cycles: 0,
            state: PromoState::default(),
        }
    }
}

impl<TTask, TTimer, TOutput> Promo<TTask, TTimer, TOutput> for BasicPromo<TTask, TTimer, TOutput>
where
    TTask: Task,
    TTimer: Timer,
    TOutput: OutputSystem<TTask, TTimer>,
{
}
