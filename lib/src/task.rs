pub trait Task<TError>: ToString + Clone {
    fn complete(&mut self) -> Result<(), TError>;
    fn is_completed(&self) -> bool;
}

#[derive(Clone, PartialEq)]
pub enum TaskKind {
    Simple(SimpleTask),
}

impl ToString for TaskKind {
    fn to_string(&self) -> String {
        match self {
            Self::Simple(task) => task.to_string(),
        }
    }
}

impl Task<()> for TaskKind {
    fn complete(&mut self) -> Result<(), ()> {
        match self {
            Self::Simple(task) => task.complete(),
        }
    }

    fn is_completed(&self) -> bool {
        match self {
            Self::Simple(task) => task.is_completed(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SimpleTask {
    completed: bool,
    description: String,
}

impl SimpleTask {
    pub fn new(description: &str) -> Self {
        Self {
            description: description.into(),
            completed: false,
        }
    }
}

impl ToString for SimpleTask {
    fn to_string(&self) -> String {
        self.description.clone()
    }
}

impl Task<()> for SimpleTask {
    fn complete(&mut self) -> Result<(), ()> {
        self.completed = true;
        Ok(())
    }

    fn is_completed(&self) -> bool {
        self.completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_complete() {
        let mut task = SimpleTask::new("Tets".into());
        assert!(!task.is_completed());

        task.complete().unwrap();
        assert!(task.is_completed());
    }
}
