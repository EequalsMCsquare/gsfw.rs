use super::Meta;
use std::fmt::Debug;

#[derive(Debug, thiserror::Error)]
pub enum Error<T> {
    #[error(
        "chain wheel duration not match. expect parent slot duration == child round duration. parent_slot: {parent_slot:?} != child_round: {child_round:?}"
    )]
    ChainDuration {
        parent_slot: std::time::Duration,
        child_round: std::time::Duration,
    },
    #[error("no wheel config to build")]
    NoWheel,
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
    DupTimer(u64),
}
