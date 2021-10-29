pub trait Task: ToString + Clone {
    fn complete(&mut self);
    fn is_completed(&self) -> bool;
}

#[derive(Clone)]
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

impl Task for TaskKind {
    fn complete(&mut self) {
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

#[derive(Clone)]
pub struct SimpleTask {
    completed: bool,
    description: String,
}

impl ToString for SimpleTask {
    fn to_string(&self) -> String {
        self.description.clone()
    }
}

impl Task for SimpleTask {
    fn complete(&mut self) {
        self.completed = true
    }

    fn is_completed(&self) -> bool {
        self.completed
    }
}
