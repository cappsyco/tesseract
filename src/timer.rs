enum Status {
    Stopped,
    Hold,
    Running,
}
struct Clock {
    time: String,
    status: Status,
}
impl Clock {
    fn default() -> Self {
        Self {
            time: String::from("00:00.00"),
            status: Status::Stopped,
        }
    }
}
