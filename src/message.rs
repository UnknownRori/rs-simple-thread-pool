use crate::Job;

#[allow(dead_code)]
pub enum Message {
    NewJob(Job),
    Terminate,
    Idle,
}
