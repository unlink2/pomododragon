use instant::Instant; // portable instant for native and wasm
use std::collections::HashMap;
use std::time::Duration;

pub trait Timer<TError>: Clone {
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

    fn default_long_break_timer() -> Self
    where
        Self: Sized,
    {
        Self::from_goal(Duration::from_secs(60 * 30))
    }

    /// start the timer
    fn start(&mut self) -> Result<(), TError>;

    /// reset is usually the same as start
    fn reset(&mut self) -> Result<(), TError> {
        self.start()
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
            Some(elapsed) => self.goal() <= elapsed && !self.is_paused(),
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

    fn is_paused(&self) -> bool;
    fn pause(&mut self);
    fn resume(&mut self);
}

/// Timer based on simple instant and duration
#[derive(Clone)]
pub struct InstantTimer {
    start: Option<Instant>,
    paused: bool,
    paused_instant: Option<Instant>,
    current_goal: Duration,
    goal: Duration,
}

impl InstantTimer {
    pub fn new(goal: Duration) -> Self {
        Self {
            start: None,
            goal,
            paused: false,
            paused_instant: None,
            current_goal: goal,
        }
    }
}

impl Timer<()> for InstantTimer {
    fn from_goal(goal: Duration) -> Self {
        Self::new(goal)
    }

    fn start(&mut self) -> Result<(), ()> {
        self.current_goal = self.goal;
        self.start = Some(Instant::now());
        Ok(())
    }

    fn elapsed(&self) -> Option<Duration> {
        self.start.map(|start| start.elapsed())
    }

    fn goal(&self) -> Duration {
        self.current_goal
    }

    fn is_paused(&self) -> bool {
        self.paused
    }

    // TODO allow pausing and unpausing
    fn pause(&mut self) {
        if !self.is_paused() {
            self.paused_instant = Some(Instant::now());
            self.paused = true;
        }
    }

    fn resume(&mut self) {
        if let Some(pause_instant) = self.paused_instant {
            self.current_goal += pause_instant.elapsed();
            self.paused = false;
            self.paused_instant = None;
        }
    }
}

/// A time string parser intended to be used for simple time input
pub struct TimeParser;
impl TimeParser {
    /// Parses a time string of form 1h5m40s100ms200us
    /// Spaces and tabs are ignored
    /// Returns as duration, or None on invalid input
    pub fn parse(time_str: &str) -> Option<Duration> {
        let mut start;
        let mut current = 0;
        let mut result = 0;

        let mut operators = HashMap::new();
        operators.insert("\0", 1000);
        operators.insert("ms", 1);
        operators.insert("s", 1000);
        operators.insert("m", 60000);
        operators.insert("h", 3600000);

        // always scan number+operator
        while current < time_str.len() {
            start = current;
            // TODO ignore white-spaces between numbers/operators
            let num = Self::scan_num(time_str, start, &mut current)?;

            start = current;
            let op = Self::scan_operator(time_str, start, &mut current, &operators)?;
            result += num * op;
        }
        Some(Duration::from_millis(result))
    }

    fn scan_operator(
        time_str: &str,
        start: usize,
        current: &mut usize,
        operators: &HashMap<&str, u64>,
    ) -> Option<u64> {
        while Self::is_alpha(time_str.chars().nth(*current).unwrap_or('\0')) {
            *current += 1;
        }
        // no operator? return 1, but only if
        // current is end of string too
        if start == *current && *current >= time_str.len() {
            Some(1)
        } else if start == *current {
            None
        } else {
            let operator = &time_str[start..*current];
            operators.get(operator).copied()
        }
    }

    fn scan_num(time_str: &str, start: usize, current: &mut usize) -> Option<u64> {
        while Self::is_numeric(time_str.chars().nth(*current).unwrap_or('\0')) {
            *current += 1;
        }

        let number = &time_str[start..*current];
        number.parse::<u64>().ok()
    }

    fn is_numeric(c: char) -> bool {
        ('0'..='9').contains(&c)
    }

    fn is_alpha(c: char) -> bool {
        ('a'..='z').contains(&c) || ('A'..='Z').contains(&c)
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

        timer.start().unwrap();
        assert!(timer.has_started());
        assert_ne!(timer.elapsed(), None);

        std::thread::sleep(Duration::from_millis(101));
        assert!(timer.is_completed());
    }

    #[test]
    fn it_should_output_percentage() {
        let mut timer = InstantTimer::new(Duration::from_millis(100));
        assert_eq!(timer.percentage(), 0.0);
        timer.start().unwrap();

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

    #[test]
    fn it_should_pause() {
        let mut timer = InstantTimer::new(Duration::from_millis(100));
        timer.start().unwrap();
        assert!(!timer.is_paused());
        assert!(timer.goal().as_millis() == 100);
        timer.pause();

        std::thread::sleep(Duration::from_millis(150));
        assert!(!timer.is_completed());
        assert!(timer.is_paused());
        timer.resume();

        assert!(timer.goal().as_millis() >= 250);
        std::thread::sleep(Duration::from_millis(150));
        assert!(timer.is_completed());
        assert!(!timer.is_paused());
    }

    #[test]
    fn it_should_parse_time_str() {
        let ms = TimeParser::parse("1h20m10s5").unwrap();
        assert_eq!(
            ms,
            Duration::from_millis((1 * 3600000) + (20 * 60000) + (10 * 1000) + 5)
        );
    }

    #[test]
    fn it_should_not_parse_time_str_bad_operator() {
        assert_eq!(TimeParser::parse("1h20m10@5"), None);
    }
}
