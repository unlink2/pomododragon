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

    /// reset is usually the same as start
    fn reset(&mut self) {
        self.start();
    }

    /// returns the seconds
    /// that passed since
    /// the timer started
    fn elapsed(&self) -> Option<Duration>;

    /// The goal of the current timer
    fn goal(&self) -> Duration;

    /// goal <= seconds
    fn is_completed(&self) -> bool {
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
            Some(elapsed) => elapsed.as_secs_f64() / self.goal().as_secs_f64(),
            None => 0.0,
        }
    }
}

/// Timer based on simple instanct and duration
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
        self.start.map(|start| start.elapsed())
    }

    fn goal(&self) -> Duration {
        self.goal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO is it really a good idea to test this
    // with sleeps?
    #[test]
    fn it_should_complete() {
        let mut timer = InstantTimer::new(Duration::from_millis(100));
        assert!(!timer.is_completed());
        assert_eq!(timer.elapsed(), None);
        assert!(!timer.has_started());

        timer.start();
        assert!(timer.has_started());
        assert_ne!(timer.elapsed(), None);

        std::thread::sleep(Duration::from_millis(101));
        assert!(timer.is_completed());
    }

    #[test]
    fn it_should_output_percentage() {
        let mut timer = InstantTimer::new(Duration::from_millis(100));
        assert_eq!(timer.percentage(), 0.0);
        timer.start();

        // we estimate the percentage since it is unclear how long sleep
        // actually takes!
        // This should *usually* not fail
        assert!(timer.percentage() > 0.0 && timer.percentage() < 0.001);

        std::thread::sleep(Duration::from_millis(40));
        assert!(timer.percentage() > 0.40 && timer.percentage() < 0.42);
        std::thread::sleep(Duration::from_millis(40));
        assert!(timer.percentage() > 0.80 && timer.percentage() < 0.82);
        std::thread::sleep(Duration::from_millis(20));
        assert!(timer.percentage() > 1.00 && timer.percentage() < 1.02);
    }
}
