use std::fmt;
use std::fmt::Display;

pub enum Error {
    AddTaskFailed,
    LocalStorageWriteFailed,
    UpdateFailed,
    PauseFailed,
    UnpauseFailed,
    StartFailed,
    ResetFailed,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::AddTaskFailed => "AddTaskFailed",
                Self::LocalStorageWriteFailed => "Local Storage Write Failed",
                Self::UpdateFailed => "Updated Failed",
                Self::PauseFailed => "Pause Failed",
                Self::UnpauseFailed => "Unpause Failed",
                Self::StartFailed => "Start Failed",
                Self::ResetFailed => "Reset Failed",
            }
        )
    }
}
