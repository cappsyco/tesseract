pub enum Status {
    Stopped,
    Hold,
    Running,
}
pub struct Clock {
    pub time: String,
    pub status: Status,
}
impl Clock {
    fn default() -> Self {
        Self {
            time: String::from("00:00.00"),
            status: Status::Stopped,
        }
    }
}
