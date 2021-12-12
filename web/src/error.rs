use std::fmt;
use std::fmt::Display;

pub enum Error {
    AddTask,
    LocalStorageWrite,
    Update,
    Pause,
    Unpause,
    Start,
    Reset,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::AddTask => "AddTaskFailed",
                Self::LocalStorageWrite => "Local Storage Write Failed",
                Self::Update => "Updated Failed",
                Self::Pause => "Pause Failed",
                Self::Unpause => "Unpause Failed",
                Self::Start => "Start Failed",
                Self::Reset => "Reset Failed",
            }
        )
    }
}
