use chrono::{DateTime, Local};
#[derive(Clone)]
pub struct ProcessData {
    pub cpu_usage: f32,
    pub mem_usage: u64,
    pub current_time: DateTime<Local>,
}
impl ProcessData {
    pub fn new(cpu_usage: f32, mem_usage: u64) -> Self {
        let current_time = chrono::offset::Local::now();
        Self {
            cpu_usage,
            mem_usage,
            current_time,
        }
    }
}
