#[derive(Debug)]
pub enum Status {
    Stopped,
    Hold,
    Ready,
    Running,
}
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Timer {
    pub time: String,
    pub status: Status,
}

impl Timer {
    pub fn default() -> Self {
        Self {
            time: String::from("00:00.00"),
            status: Status::Stopped,
        }
    }
    pub fn start(&mut self) {
        self.status = Status::Running;
    }
    pub fn stop(&mut self) {
        self.status = Status::Stopped;
    }
    pub fn hold(&mut self) {
        self.status = Status::Hold;
    }
}
