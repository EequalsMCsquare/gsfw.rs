use std::fmt::Debug;

use super::timer::Meta;

#[derive(Debug)]
pub(crate) enum TimeWheelProto<T> {
    Tick,
    Add(Meta<T>),
    /// BatchAdd require all metas' slot is the same
    BatchAdd(Vec<(Meta<T>, usize)>),
    Cancel {
        id: u64,
        slot_hint: usize,
    },
    Accelerate {
        id: u64,
        slot_hint: usize,
        dur: std::time::Duration,
    },
    Delay {
        id: u64, 
        slot_hint: usize, 
        dur: std::time::Duration
    },
    Trigger {
        id: u64, 
        slot_hint: usize
    }

}
