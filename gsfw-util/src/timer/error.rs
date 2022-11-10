use std::fmt::Debug;
use super::Meta;

#[derive(Debug, thiserror::Error)]
pub enum Error<T> 
{
    #[error("attempt to poll a finished timer")]
    TimerFinish,
    #[error("trigger time is already elapsed")]
    TimeElapse(Option<T>),
    #[error("trigger time overflow")]
    Overflow(Option<T>),
    #[error("time wheel channel close")]
    Channel(Option<T>),
    #[error("time wheel channel close")]
    BatchChannel(Vec<Meta<T>>),
    #[error("timer {0} not found")]
    NoRecord(u64),
    #[error("duplicated timer {0}")]
    DupTimer(u64)
}