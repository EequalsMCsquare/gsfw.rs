use super::{timer::Meta, tw_proto::TimeWheelProto};
use futures::{ready, Future, FutureExt};
use parking_lot::RwLock;
use pin_project::pin_project;
use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    ops::Add,
    pin::Pin,
    sync::Arc,
    task::Poll,
};
use tokio::sync::mpsc;

///
/// # Time Wheel Proxy
/// this provide user with easy-use functions to interact with TimeWheel
/// # example
/// ```rust
/// use gsfw_util::timer::Wheel;
/// #[tokio::main]
/// async fn main() {
///     let now = std::time::Instant::now();
///     let mut wheel = Wheel::<i32>::new(60, std::time::Duration::from_secs(1), now);
///     let snapshot = wheel.dispatch(std::time::Duration::from_secs(3), 1).await.unwrap();
///     let recv_data = wheel.tick().await;
///     assert_eq!(recv_data.len(), 1);
///     assert_eq!(recv_data[0].id, snapshot.id);
///     assert_eq!(recv_data[0].data, Some(1));
///     assert_eq!(recv_data[0].start, snapshot.start);
///     assert_eq!(recv_data[0].end, snapshot.end);
/// }
/// ```
pub struct WheelProxy<T>
where
    T: Debug + Send,
{
    slot: u32,
    slot_duration: std::time::Duration,
    start: Arc<RwLock<std::time::Instant>>,
    tick_rx: mpsc::Receiver<VecDeque<Meta<T>>>,
    inner_tx: mpsc::Sender<TimeWheelProto<T>>,
    timer_map: HashMap<u64, super::Snapshot>,

    ticker_join: tokio::task::JoinHandle<()>,
    inner_join: tokio::task::JoinHandle<()>,
}

impl<T> WheelProxy<T>
where
    T: Send + Debug + 'static,
{
    pub fn new(slot: u32, slot_duration: std::time::Duration, start: std::time::Instant) -> Self {
        Inner::new(slot, slot_duration, start)
    }

    pub fn slot(&self) -> u32 {
        self.slot
    }

    pub fn slot_duration(&self) -> std::time::Duration {
        self.slot_duration.clone()
    }

    pub fn round_duration(&self) -> std::time::Duration {
        self.slot_duration * self.slot
    }

    pub fn round_end(&self) -> std::time::Instant {
        self.start.read().add(self.slot * self.slot_duration)
    }

    pub async fn dispatch(
        &mut self,
        duration: std::time::Duration,
        data: T,
    ) -> Result<super::Snapshot, super::Error<T>> {
        self.dispatch_until(std::time::Instant::now() + duration, data)
            .await
    }

    pub async fn dispatch_until(
        &mut self,
        end: std::time::Instant,
        data: T,
    ) -> Result<super::Snapshot, super::Error<T>> {
        let now = std::time::Instant::now();
        if end < now {
            return Err(super::Error::TimeElapse(Some(data)));
        }
        if end > self.round_end() {
            return Err(super::Error::Overflow(Some(data)));
        }
        let meta = Meta::new(now, end, data);
        let snapshot = super::Snapshot {
            id: meta.id,
            start: meta.start,
            end,
        };
        let timer_id = meta.id;
        if let Err(err) = self.inner_tx.send(TimeWheelProto::Add(meta)).await {
            return Err(match err.0 {
                TimeWheelProto::Add(meta) => super::Error::Channel(meta.data),
                // this shall never happen
                _ => panic!("unexpected error"),
            });
        }
        self.timer_map.insert(
            timer_id,
            super::Snapshot {
                id: snapshot.id,
                start: snapshot.start,
                end,
            },
        );
        return Ok(snapshot);
    }

    pub async fn cancel(&mut self, id: u64) -> Result<(), super::Error<T>> {
        let snapshot = self
            .timer_map
            .remove(&id)
            .ok_or(super::Error::NoRecord(id))?;
        let slot = find_slot(
            self.start.read().clone(),
            self.slot_duration.as_nanos(),
            snapshot.end,
        );
        self.inner_tx
            .send(TimeWheelProto::Cancel {
                id,
                slot_hint: slot as usize,
            })
            .await
            .map_err(|_| super::Error::Channel(None))
    }

    pub async fn accelerate(
        &mut self,
        id: u64,
        acc_duration: std::time::Duration,
    ) -> Result<(), super::Error<T>> {
        let now = std::time::Instant::now();
        // check timer exist
        let snapshot = self
            .timer_map
            .get_mut(&id)
            .ok_or(super::Error::NoRecord(id))?;
        let slot = find_slot(
            self.start.read().clone(),
            self.slot_duration.as_nanos(),
            snapshot.end,
        );
        // trigger now -> Trigger
        if snapshot.end - acc_duration < now {
            self.inner_tx
                .send(TimeWheelProto::Trigger {
                    id,
                    slot_hint: slot as usize,
                })
                .await
                .map_err(|_| super::Error::Channel(None))?;
            snapshot.end -= acc_duration;
            return Ok(());
        }
        // future trigger -> Accelerate
        self.inner_tx
            .send(TimeWheelProto::Accelerate {
                id,
                slot_hint: slot as usize,
                dur: acc_duration,
            })
            .await
            .map_err(|_| super::Error::Channel(None))
    }

    pub async fn delay(
        &mut self,
        id: u64,
        delay_duration: std::time::Duration,
    ) -> Result<(), super::Error<T>> {
        let snapshot = self
            .timer_map
            .get_mut(&id)
            .ok_or(super::Error::NoRecord(id))?;
        let round_start = self.start.read().clone();
        // check overflow
        if snapshot.end + delay_duration > round_start + self.slot_duration * self.slot {
            return Err(super::Error::Overflow(None));
        }
        let slot = find_slot(round_start, self.slot_duration.as_nanos(), snapshot.end);
        self.inner_tx
            .send(TimeWheelProto::Delay {
                id,
                slot_hint: slot as usize,
                dur: delay_duration,
            })
            .await
            .map_err(|_| super::Error::Channel(None))
    }

    pub async fn trigger(&mut self, id: u64) -> Result<(), super::Error<T>> {
        let snapshot = self
            .timer_map
            .get_mut(&id)
            .ok_or(super::Error::NoRecord(id))?;
        let slot = find_slot(
            self.start.read().clone(),
            self.slot_duration.as_nanos(),
            snapshot.end,
        );
        self.inner_tx
            .send(TimeWheelProto::Trigger {
                id,
                slot_hint: slot as usize,
            })
            .await
            .map_err(|_| super::Error::Channel(None))
    }

    /// batch_add atomic operation. either all metas are added to the wheel, nor none is added.
    /// batch_add will first check metas's id is unique and
    /// then check all metas are not overflow
    pub async fn batch_add(
        &mut self,
        metas: Vec<Meta<T>>,
    ) -> Result<Vec<super::Snapshot>, super::Error<T>> {
        if metas.len() == 0 {
            return Ok(Vec::new());
        }

        let start = self.start.read().clone();
        let round_end = start.add(self.round_duration());
        let slot_dur = self.slot_duration.as_nanos();
        let mut proto = Vec::with_capacity(metas.len());
        for mut meta in metas {
            // check id unique
            if self.timer_map.contains_key(&meta.id) {
                return Err(super::Error::DupTimer(meta.id));
            }
            // ensure not overflow and elapse
            if meta.start < start {
                return Err(super::Error::TimeElapse(meta.data.take()));
            } else if meta.end > round_end {
                return Err(super::Error::Overflow(meta.data.take()));
            } else {
                // let diff = (meta.end - start).as_nanos();
                let meta_end = meta.end;
                proto.push((
                    meta,
                    // (diff / slot_dur - if diff % slot_dur != 0 { 0 } else { 1 }) as usize,
                    find_slot(start, slot_dur, meta_end) as usize,
                ));
            }
        }
        // insert snapshot of this batch timers
        proto.iter().for_each(|(meta, _)| {
            self.timer_map.insert(
                meta.id,
                super::Snapshot {
                    id: meta.id,
                    start: meta.start,
                    end: meta.end,
                },
            );
        });
        let ret = proto
            .iter()
            .map(|(meta, _)| super::Snapshot {
                id: meta.id,
                start: meta.start,
                end: meta.end,
            })
            .collect();
        if let Err(err) = self.inner_tx.send(TimeWheelProto::BatchAdd(proto)).await {
            return Err(match err.0 {
                TimeWheelProto::BatchAdd(batch) => {
                    super::Error::BatchChannel(batch.into_iter().map(|(meta, _)| meta).collect())
                }
                // this shall never happen
                _ => panic!("unexpected error"),
            });
        }
        return Ok(ret);
    }

    pub async fn tick(&mut self) -> Vec<Meta<T>> {
        if let Some(metas) = self.tick_rx.recv().await {
            return metas
                .into_iter()
                .filter(|meta| {
                    if self.timer_map.get(&meta.id).is_some() {
                        self.timer_map.remove(&meta.id);
                        return true;
                    }
                    return false;
                })
                .collect();
        }
        panic!()
    }
}

impl<T> Drop for WheelProxy<T>
where
    T: Debug + Send,
{
    fn drop(&mut self) {
        self.inner_join.abort();
        self.ticker_join.abort();
    }
}

enum InnerState {
    PollRecv,
    SendTick(Pin<Box<dyn Future<Output = ()> + Send>>),
}

#[pin_project]
struct Inner<T: Debug + Send> {
    pub(crate) slot: u32,
    pub(crate) slot_duration: std::time::Duration,
    pub(crate) wq: VecDeque<VecDeque<Meta<T>>>,
    start: Arc<RwLock<std::time::Instant>>,
    tx: mpsc::Sender<TimeWheelProto<T>>,
    rx: mpsc::Receiver<TimeWheelProto<T>>,
    tick_tx: mpsc::Sender<VecDeque<Meta<T>>>,
    state: InnerState,
}

impl<T> Inner<T>
where
    T: Send + Debug + 'static,
{
    fn new(
        slot: u32,
        slot_duration: std::time::Duration,
        start: std::time::Instant,
    ) -> WheelProxy<T> {
        let (tx, rx) = mpsc::channel(4);
        let (tick_tx, tick_rx) = mpsc::channel(64);
        let mut wq = VecDeque::with_capacity(slot as usize);
        wq.resize_with(slot as usize, Default::default);
        let arc_start = Arc::new(RwLock::new(start));
        let inner = Self {
            slot,
            slot_duration: slot_duration.clone(),
            start: arc_start.clone(),
            wq,
            tx: tx.clone(),
            rx,
            tick_tx,
            state: InnerState::PollRecv,
        };
        let inner_tx = tx.clone();
        let ticker_join = tokio::spawn(async move {
            let mut interval =
                tokio::time::interval_at(start.clone().into(), inner.slot_duration.clone());
            let tx = inner_tx.clone();
            loop {
                interval.tick().await;
                tx.send(TimeWheelProto::Tick).await.unwrap();
            }
        });

        WheelProxy {
            tick_rx,
            inner_tx: tx,
            ticker_join,
            inner_join: tokio::spawn(inner),
            slot,
            slot_duration,
            start: arc_start,
            timer_map: Default::default(),
        }
    }
}

impl<T: Debug + Send> Future for Inner<T>
where
    T: Send + 'static,
{
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        loop {
            match this.state {
                InnerState::PollRecv => {
                    if let Some(proto) = ready!(this.rx.poll_recv(cx)) {
                        match proto {
                            TimeWheelProto::Tick => {
                                if let Some(metas) = this.wq.pop_front() {
                                    this.wq.push_back(Default::default());
                                    if metas.len() == 0 {
                                        continue;
                                    }
                                    let tx = this.tick_tx.clone();
                                    let mut fut = Box::pin(async move {
                                        if let Err(err) = tx.send(metas).await {
                                            tracing::error!("TickTx send error: {:?}", err);
                                        }
                                    });
                                    // try poll immediately
                                    if let Poll::Pending = fut.poll_unpin(cx) {
                                        *this.state = InnerState::SendTick(fut);
                                        // yield
                                        return Poll::Pending;
                                    }
                                }
                            }
                            TimeWheelProto::Add(meta) => {
                                let start = this.start.read().clone();
                                let slot_dur = this.slot_duration.as_nanos();
                                let slot = find_slot(start, slot_dur, meta.end);
                                if slot as u32 >= *this.slot {
                                    tracing::error!("[Add] overflow timer. {:?}", meta);
                                    continue;
                                }
                                tracing::info!("[Add] add timer. {:?}", meta);
                                this.wq.get_mut(slot as usize).unwrap().push_back(meta);
                            }
                            TimeWheelProto::BatchAdd(batch) => {
                                for (meta, slot) in batch {
                                    let vec = this.wq.get_mut(slot).unwrap();
                                    tracing::info!("[BatchAdd] add timer. {:?}", meta);
                                    vec.push_back(meta);
                                }
                            }
                            TimeWheelProto::Cancel { id, slot_hint } => {
                                let vec = this.wq.get_mut(slot_hint).unwrap();
                                tracing::info!("[Cancel] cancel timer {}", id);
                                if let Some((idx, _)) =
                                    vec.iter().enumerate().find(|(_, meta)| meta.id == id)
                                {
                                    vec.swap_remove_back(idx);
                                } else {
                                    tracing::warn!("[Cancel] timer {} not found", id);
                                }
                            }
                            TimeWheelProto::Accelerate { id, slot_hint, dur } => {
                                let vec = this.wq.get_mut(slot_hint).unwrap();
                                if let Some((idx, meta)) =
                                    vec.iter_mut().enumerate().find(|(_, meta)| meta.id == id)
                                {
                                    let new_slot = find_slot(
                                        this.start.read().clone(),
                                        this.slot_duration.as_nanos(),
                                        meta.end - dur,
                                    ) as usize;
                                    meta.end -= dur;
                                    // before current slot
                                    if new_slot < slot_hint {
                                        let meta = vec.remove(idx).unwrap();
                                        this.wq.get_mut(new_slot).unwrap().push_back(meta);
                                    }
                                } else {
                                    tracing::warn!("[Accelerate] timer {} not found", id);
                                }
                            }
                            TimeWheelProto::Delay { id, slot_hint, dur } => {
                                let vec = this.wq.get_mut(slot_hint).unwrap();
                                if let Some((idx, meta)) =
                                    vec.iter_mut().enumerate().find(|(_, meta)| meta.id == id)
                                {
                                    let new_slot = find_slot(
                                        this.start.read().clone(),
                                        this.slot_duration.as_nanos(),
                                        meta.end + dur,
                                    ) as usize;
                                    meta.end += dur;
                                    // after current slot
                                    if new_slot > slot_hint {
                                        let meta = vec.remove(idx).unwrap();
                                        this.wq.get_mut(new_slot).unwrap().push_back(meta);
                                    }
                                } else {
                                    tracing::warn!("[Delay] timer {} not found", id);
                                }
                            }
                            TimeWheelProto::Trigger { id, slot_hint } => {
                                let now = std::time::Instant::now();
                                let vec = this.wq.get_mut(slot_hint).unwrap();
                                tracing::info!("[Trigger] trigger timer {} now", id);
                                if let Some((idx, _)) =
                                    vec.iter().enumerate().find(|(_, meta)| meta.id == id)
                                {
                                    if let Some(mut meta) = vec.swap_remove_back(idx) {
                                        meta.end = now;
                                        let mut metas = VecDeque::with_capacity(1);
                                        metas.push_back(meta);
                                        let tx = this.tick_tx.clone();
                                        let mut fut = Box::pin(async move {
                                            if let Err(err) = tx.send(metas).await {
                                                tracing::error!("TickTx send error: {:?}", err);
                                            }
                                        });
                                        // try poll immediately
                                        if let Poll::Pending = fut.poll_unpin(cx) {
                                            *this.state = InnerState::SendTick(fut);
                                            // yield
                                            return Poll::Pending;
                                        }
                                    } else {
                                        tracing::warn!("[Trigger] trigger timer {} not found", id);
                                    }
                                } else {
                                    tracing::warn!("[Cancel] timer {} not found", id);
                                }
                            }
                        }
                    }
                }
                InnerState::SendTick(fut) => match fut.poll_unpin(cx) {
                    std::task::Poll::Ready(_) => *this.state = InnerState::PollRecv,
                    std::task::Poll::Pending => return Poll::Pending,
                },
            }
        }
    }
}

#[inline(always)]
pub(crate) fn find_slot(
    wheel_start: std::time::Instant,
    slot_dur_ns: u128,
    timer_end: std::time::Instant,
) -> u128 {
    let diff = (timer_end - wheel_start).as_nanos();
    diff / slot_dur_ns - if diff % slot_dur_ns != 0 { 0 } else { 1 }
}
