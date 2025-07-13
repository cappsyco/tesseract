use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Stopped,
    Hold,
    Ready,
    Running,
}

#[derive(Debug)]
pub struct Timer {
    pub time: u64,
    pub status: Status,
}

impl Timer {
    pub fn default() -> Self {
        Self {
            time: 0,
            status: Status::Stopped,
        }
    }
    pub fn display(&self) -> String {
        let duration = Duration::from_millis(self.time);
        let minutes = duration.as_millis() / 60_000;
        let seconds = (duration.as_millis() % 60_000) / 1_000;
        let millis = (duration.as_millis() % 1_000) / 10;

        if minutes > 0 {
            format!("{}:{:02}.{:02}", minutes, seconds, millis)
        } else {
            format!("{}.{:02}", seconds, millis)
        }
    }
    pub fn start(&mut self) {
        self.status = Status::Running;
    }
}
