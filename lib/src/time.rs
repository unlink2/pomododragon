use std::time::{Duration, Instant};

pub trait Timer {
    fn from_goal(goal: Duration) -> Self;

    fn default_work_timer() -> Self
    where
        Self: Sized,
    {
        Self::from_goal(Duration::from_secs(60 * 25))
    }

    fn default_break_timer() -> Self
    where
        Self: Sized,
    {
        Self::from_goal(Duration::from_secs(60 * 5))
    }

    fn default_rest_timer() -> Self
    where
        Self: Sized,
    {
        Self::from_goal(Duration::from_secs(60 * 30))
    }

    /// start the timer
    fn start(&mut self);

    /// returns the seconds
    /// that passed since
    /// the timer started
    fn elapsed(&self) -> Option<Duration>;

    /// The goal of the current timer
    fn goal(&self) -> Duration;

    /// goal <= seconds
    fn is_over(&self) -> bool {
        match self.elapsed() {
            Some(elapsed) => self.goal() <= elapsed,
            None => false,
        }
    }

    fn has_started(&self) -> bool {
        self.elapsed() != None
    }

    fn percentage(&self) -> f64 {
        match self.elapsed() {
            Some(elapsed) => elapsed.as_secs() as f64 / self.goal().as_secs() as f64,
            None => 0.0,
        }
    }
}

pub struct InstantTimer {
    start: Option<Instant>,
    goal: Duration,
}

impl InstantTimer {
    pub fn new(goal: Duration) -> Self {
        Self { start: None, goal }
    }
}

impl Timer for InstantTimer {
    fn from_goal(goal: Duration) -> Self {
        Self::new(goal)
    }

    fn start(&mut self) {
        self.start = Some(Instant::now())
    }

    fn elapsed(&self) -> Option<Duration> {
        match self.start {
            Some(start) => Some(start.elapsed()),
            None => None,
        }
    }

    fn goal(&self) -> Duration {
        self.goal
    }
}
