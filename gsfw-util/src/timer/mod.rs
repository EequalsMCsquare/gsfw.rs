mod error;
pub mod timer;
mod tw_proto;
mod wheel;
use error::Error;
pub use timer::{Meta, Snapshot, Timer};
pub use wheel::WheelProxy as Wheel;

#[cfg(test)]
mod test {
    use super::*;
    use std::time::{Duration, Instant};
    use test_case::test_case;

    const TOLERANCE: Duration = Duration::from_millis(50);

    fn build_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    #[test_case(Instant::now() + Duration::from_millis(50), 1; "when timer trigger in 50 ms")]
    #[test_case(Instant::now() + Duration::from_millis(100), 2; "when timer trigger in 100 ms")]
    #[test_case(Instant::now() + Duration::from_millis(200), 2; "when timer trigger in 200 ms")]
    #[test_case(Instant::now() + Duration::from_secs(1), 2; "when timer trigger in 1 sec")]
    fn test_timer(end: Instant, data: i32) {
        let rt = build_runtime();
        rt.block_on(async {
            let now = Instant::now();
            let ret = Timer::new(now, end, data).await;
            assert_eq!(ret.unwrap(), data);
            assert!(Instant::now() > end);
        });
    }

    #[test_case(60, Duration::from_secs(1); "minute wheel with 1 sec per slot")]
    #[test_case(30, Duration::from_secs(2); "minute wheel with 2 sec per slot")]
    #[test_case(20, Duration::from_secs(3); "minute wheel with 3 sec per slot")]
    #[test_case(15, Duration::from_secs(4); "minute wheel with 4 sec per slot")]
    #[test_case(12, Duration::from_secs(5); "minute wheel with 5 sec per slot")]
    #[test_case(10, Duration::from_secs(6); "minute wheel with 6 sec per slot")]
    fn test_min_wheel_create(slot: u32, slot_duration: Duration) {
        let rt = build_runtime();
        rt.block_on(async {
            let now = Instant::now();
            let wheel = Wheel::<()>::new(slot, slot_duration, now);
            assert_eq!(wheel.slot(), slot);
            assert_eq!(wheel.slot_duration(), slot_duration);
            assert_eq!(wheel.round_duration(), Duration::from_secs(60));
            assert_eq!(wheel.round_duration(), slot * slot_duration);
            let before_end = wheel.round_end();
            assert_eq!(before_end, now + wheel.round_duration());
            tokio::time::sleep(slot_duration).await;
            assert!(wheel.round_end() >= before_end + slot_duration);
        })
    }

    #[test_case(125, Duration::from_millis(8); "second wheel with 8 ms per slot")]
    #[test_case(100, Duration::from_millis(10); "second wheel with 10 ms per slot")]
    #[test_case(50, Duration::from_millis(20); "second wheel with 20 ms per slot")]
    #[test_case(20, Duration::from_millis(50); "second wheel with 50 ms per slot")]
    #[test_case(10, Duration::from_millis(100); "second wheel with 100 ms per slot")]
    #[test_case(8, Duration::from_millis(125); "second wheel with 125 ms per slot")]
    #[test_case(5, Duration::from_millis(200); "second wheel with 200 ms per slot")]
    #[test_case(4, Duration::from_millis(250); "second wheel with 250 ms per slot")]
    #[test_case(2, Duration::from_millis(500); "second wheel with 500 ms per slot")]
    fn test_sec_wheel_create(slot: u32, slot_duration: Duration) {
        let rt = build_runtime();
        rt.block_on(async {
            let now = Instant::now();
            let wheel = Wheel::<()>::new(slot, slot_duration, now);
            assert_eq!(wheel.slot(), slot);
            assert_eq!(wheel.slot_duration(), slot_duration);
            assert_eq!(wheel.round_duration(), Duration::from_secs(1));
            assert_eq!(wheel.round_duration(), slot * slot_duration);
            let before_end = wheel.round_end();
            assert_eq!(before_end, now + wheel.round_duration());
            tokio::time::sleep(slot_duration).await;
            assert!(wheel.round_end() >= before_end + slot_duration);
        })
    }

    // minute wheel
    #[test_case(60, Duration::from_secs(1), Duration::from_secs_f32(0.999), 1, 0)]
    #[test_case(30, Duration::from_secs(2), Duration::from_secs_f32(1.999), 1, 0)]
    #[test_case(20, Duration::from_secs(3), Duration::from_secs_f32(2.999), 1, 0)]
    #[test_case(15, Duration::from_secs(4), Duration::from_secs_f32(3.999), 1, 0)]
    #[test_case(12, Duration::from_secs(5), Duration::from_secs_f32(4.999), 1, 0)]
    #[test_case(10, Duration::from_secs(6), Duration::from_secs_f32(5.999), 1, 0)]
    #[test_case(60, Duration::from_secs(1), Duration::from_secs_f32(1.999), 1, 1)]
    #[test_case(30, Duration::from_secs(2), Duration::from_secs_f32(2.999), 1, 1)]
    #[test_case(20, Duration::from_secs(3), Duration::from_secs_f32(3.999), 1, 1)]
    #[test_case(15, Duration::from_secs(4), Duration::from_secs_f32(4.999), 1, 1)]
    #[test_case(12, Duration::from_secs(5), Duration::from_secs_f32(5.999), 1, 1)]
    #[test_case(10, Duration::from_secs(6), Duration::from_secs_f32(6.999), 1, 1)]
    #[test_case(60, Duration::from_secs(1), Duration::from_secs_f32(2.999), 1, 2)]
    #[test_case(30, Duration::from_secs(2), Duration::from_secs_f32(4.999), 1, 2)]
    #[test_case(20, Duration::from_secs(3), Duration::from_secs_f32(6.999), 1, 2)]
    #[test_case(15, Duration::from_secs(4), Duration::from_secs_f32(8.999), 1, 2)]
    #[test_case(12, Duration::from_secs(5), Duration::from_secs_f32(10.999), 1, 2)]
    #[test_case(10, Duration::from_secs(6), Duration::from_secs_f32(12.999), 1, 2)]
    fn test_wheel_dispatch_until_tick_n(
        slot: u32,
        slot_duration: Duration,
        append: Duration,
        data: i32,
        tick_n: u32,
    ) {
        let rt = build_runtime();
        rt.block_on(async {
            let now = Instant::now();
            // let slot_duration = Duration::from_secs(60) / slot;
            let mut wheel = Wheel::<i32>::new(slot, slot_duration, now);
            let snapshot = wheel.dispatch_until(now + append, data).await;
            assert!(snapshot.is_ok());
            let snapshot = snapshot.unwrap();
            let elapse_timers = wheel.tick().await;
            assert!(Instant::now() >= now + slot_duration * tick_n);
            assert!(Instant::now() <= now + slot_duration * tick_n + TOLERANCE);
            assert_eq!(elapse_timers.len(), 1);
            assert_eq!(elapse_timers[0].id, snapshot.id);
            assert_eq!(elapse_timers[0].data, Some(data));
            assert_eq!(elapse_timers[0].start, snapshot.start);
            assert_eq!(elapse_timers[0].end, snapshot.end);
        });
    }

    // // minute wheel
    #[test_case(60, Duration::from_secs(1), Duration::from_secs_f32(0.999), 1, 0)]
    #[test_case(30, Duration::from_secs(2), Duration::from_secs_f32(1.999), 1, 0)]
    #[test_case(20, Duration::from_secs(3), Duration::from_secs_f32(2.999), 1, 0)]
    #[test_case(15, Duration::from_secs(4), Duration::from_secs_f32(3.999), 1, 0)]
    #[test_case(12, Duration::from_secs(5), Duration::from_secs_f32(4.999), 1, 0)]
    #[test_case(10, Duration::from_secs(6), Duration::from_secs_f32(5.999), 1, 0)]
    #[test_case(60, Duration::from_secs(1), Duration::from_secs_f32(1.999), 1, 1)]
    #[test_case(30, Duration::from_secs(2), Duration::from_secs_f32(2.999), 1, 1)]
    #[test_case(20, Duration::from_secs(3), Duration::from_secs_f32(3.999), 1, 1)]
    #[test_case(15, Duration::from_secs(4), Duration::from_secs_f32(4.999), 1, 1)]
    #[test_case(12, Duration::from_secs(5), Duration::from_secs_f32(5.999), 1, 1)]
    #[test_case(10, Duration::from_secs(6), Duration::from_secs_f32(6.999), 1, 1)]
    #[test_case(60, Duration::from_secs(1), Duration::from_secs_f32(2.999), 1, 2)]
    #[test_case(30, Duration::from_secs(2), Duration::from_secs_f32(4.999), 1, 2)]
    #[test_case(20, Duration::from_secs(3), Duration::from_secs_f32(6.999), 1, 2)]
    #[test_case(15, Duration::from_secs(4), Duration::from_secs_f32(8.999), 1, 2)]
    #[test_case(12, Duration::from_secs(5), Duration::from_secs_f32(10.999), 1, 2)]
    #[test_case(10, Duration::from_secs(6), Duration::from_secs_f32(12.999), 1, 2)]
    fn test_wheel_dispatch_tick_n(
        slot: u32,
        slot_duration: Duration,
        duration: Duration,
        data: i32,
        tick_n: u32,
    ) {
        let rt = build_runtime();
        rt.block_on(async {
            let now = Instant::now();
            // let slot_duration = Duration::from_secs(60) / slot;
            let mut wheel = Wheel::<i32>::new(slot, slot_duration, now);
            let snapshot = wheel.dispatch(duration, data).await;
            assert!(snapshot.is_ok());
            let snapshot = snapshot.unwrap();
            let elapse_timers = wheel.tick().await;
            assert!(Instant::now() >= now + slot_duration * tick_n);
            assert!(Instant::now() <= now + slot_duration * tick_n + TOLERANCE);
            assert_eq!(elapse_timers.len(), 1);
            assert_eq!(elapse_timers[0].id, snapshot.id);
            assert_eq!(elapse_timers[0].data, Some(data));
            assert_eq!(elapse_timers[0].start, snapshot.start);
            assert_eq!(elapse_timers[0].end, snapshot.end);
        });
    }

    #[test_case(60, Duration::from_secs(1), Duration::from_secs_f32(0.999), vec![1,2,3], 0)]
    #[test_case(30, Duration::from_secs(2), Duration::from_secs_f32(1.999), vec![1,2,3], 0)]
    #[test_case(20, Duration::from_secs(3), Duration::from_secs_f32(2.999), vec![1,2,3], 0)]
    #[test_case(15, Duration::from_secs(4), Duration::from_secs_f32(3.999), vec![1,2,3], 0)]
    #[test_case(12, Duration::from_secs(5), Duration::from_secs_f32(4.999), vec![1,2,3], 0)]
    #[test_case(10, Duration::from_secs(6), Duration::from_secs_f32(5.999), vec![1,2,3], 0)]
    #[test_case(60, Duration::from_secs(1), Duration::from_secs_f32(1.999), vec![1,2,3], 1)]
    #[test_case(30, Duration::from_secs(2), Duration::from_secs_f32(2.999), vec![1,2,3], 1)]
    #[test_case(20, Duration::from_secs(3), Duration::from_secs_f32(3.999), vec![1,2,3], 1)]
    #[test_case(15, Duration::from_secs(4), Duration::from_secs_f32(4.999), vec![1,2,3], 1)]
    #[test_case(12, Duration::from_secs(5), Duration::from_secs_f32(5.999), vec![1,2,3], 1)]
    #[test_case(10, Duration::from_secs(6), Duration::from_secs_f32(6.999), vec![1,2,3], 1)]
    #[test_case(60, Duration::from_secs(1), Duration::from_secs_f32(2.999), vec![1,2,3], 2)]
    #[test_case(30, Duration::from_secs(2), Duration::from_secs_f32(4.999), vec![1,2,3], 2)]
    #[test_case(20, Duration::from_secs(3), Duration::from_secs_f32(6.999), vec![1,2,3], 2)]
    #[test_case(15, Duration::from_secs(4), Duration::from_secs_f32(8.999), vec![1,2,3], 2)]
    #[test_case(12, Duration::from_secs(5), Duration::from_secs_f32(10.999), vec![1,2,3], 2)]
    #[test_case(10, Duration::from_secs(6), Duration::from_secs_f32(12.999), vec![1,2,3], 2)]
    fn test_wheel_dispatch_multi(
        slot: u32,
        slot_duration: Duration,
        duration: Duration,
        data_list: Vec<i32>,
        tick_n: u32,
    ) {
        let rt = build_runtime();
        rt.block_on(async {
            let now = Instant::now();
            let mut wheel = Wheel::<i32>::new(slot, slot_duration, now);
            let mut snapshot_list = Vec::with_capacity(data_list.len());
            for data in &data_list {
                let snapshot = wheel.dispatch(duration, data.clone()).await;
                assert!(snapshot.is_ok());
                snapshot_list.push(snapshot.unwrap());
            }
            let elapse_timers = wheel.tick().await;
            assert_eq!(elapse_timers.len(), data_list.len());
            assert!(Instant::now() >= now + slot_duration * tick_n);
            assert!(Instant::now() <= now + slot_duration * tick_n + TOLERANCE);
            for (idx, meta) in elapse_timers.iter().enumerate() {
                assert_eq!(meta.id, snapshot_list[idx].id);
                assert_eq!(meta.data, Some(data_list[idx]));
                assert_eq!(meta.start, snapshot_list[idx].start);
                assert_eq!(meta.end, snapshot_list[idx].end);
            }
        });
    }

    #[test_case(60, Duration::from_secs(1), Duration::from_secs_f32(0.999), 1, 0)]
    #[test_case(60, Duration::from_secs(1), Duration::from_secs_f32(1.500), 1, 1)]
    #[test_case(30, Duration::from_secs(2), Duration::from_secs_f32(1.999), 1, 0)]
    #[test_case(30, Duration::from_secs(2), Duration::from_secs_f32(3.500), 1, 1)]
    #[test_case(4, Duration::from_millis(250), Duration::from_millis(200), 1, 0)]
    #[test_case(4, Duration::from_millis(250), Duration::from_millis(400), 1, 1)]
    #[test_case(4, Duration::from_millis(250), Duration::from_millis(600), 1, 2)]
    #[test_case(20, Duration::from_millis(50), Duration::from_millis(40), 1, 0)]
    #[test_case(20, Duration::from_millis(50), Duration::from_millis(90), 1, 1)]
    #[test_case(20, Duration::from_millis(50), Duration::from_millis(140), 1, 2)]
    fn test_wheel_cancel(
        slot: u32,
        slot_duration: Duration,
        duration: Duration,
        data: i32,
        tick_n: u32,
    ) {
        let rt = build_runtime();
        rt.block_on(async {
            let now = Instant::now();
            let mut wheel = Wheel::<i32>::new(slot, slot_duration, now);
            let snapshot = wheel.dispatch(duration, data).await;
            assert!(snapshot.is_ok());
            let snapshot = snapshot.unwrap();
            assert!(wheel.cancel(snapshot.id).await.is_ok());
            let timeout_ret =
                tokio::time::timeout(slot_duration * (tick_n + 1), wheel.tick()).await;
            assert!(timeout_ret.is_err());
        });
    }

    #[test_case(
        60,
        Duration::from_secs(1),
        Duration::from_secs_f32(0.999),
        1,
        Duration::from_millis(200)
    )]
    #[test_case(
        30,
        Duration::from_secs(2),
        Duration::from_secs_f32(1.999),
        1,
        Duration::from_millis(200)
    )]
    #[test_case(
        100,
        Duration::from_millis(10),
        Duration::from_millis(50),
        1,
        Duration::from_millis(100)
    )]
    #[test_case(
        100,
        Duration::from_millis(10),
        Duration::from_millis(50),
        1,
        Duration::from_millis(200)
    )]
    #[test_case(
        10,
        Duration::from_millis(100),
        Duration::from_millis(200),
        1,
        Duration::from_millis(300)
    )]
    #[test_case(
        5,
        Duration::from_millis(200),
        Duration::from_millis(380),
        1,
        Duration::from_millis(40)
    )]
    fn test_wheel_delay(
        slot: u32,
        slot_duration: Duration,
        duration: Duration,
        data: i32,
        delay: Duration,
    ) {
        let rt = build_runtime();
        rt.block_on(async {
            let now = Instant::now();
            let mut wheel = Wheel::<i32>::new(slot, slot_duration, now);
            let snapshot = wheel.dispatch(duration, data).await;
            assert!(snapshot.is_ok());
            let snapshot = snapshot.unwrap();
            assert!(wheel.delay(snapshot.id, delay).await.is_ok());
            let timeout_ret = tokio::time::timeout(
                (delay.as_nanos() / slot_duration.as_nanos()) as u32 * slot_duration,
                wheel.tick(),
            )
            .await;
            assert!(timeout_ret.is_err());
        });
    }

    #[test_case(60, Duration::from_secs(1), Duration::from_secs(1), 1)]
    #[test_case(60, Duration::from_secs(1), Duration::from_secs(59), 1)]
    #[test_case(5, Duration::from_millis(200), Duration::from_millis(201), 1)]
    #[test_case(5, Duration::from_millis(200), Duration::from_millis(999), 1)]
    fn test_wheel_trigger(slot: u32, slot_duration: Duration, duration: Duration, data: i32) {
        let rt = build_runtime();
        rt.block_on(async {
            let now = Instant::now();
            let mut wheel = Wheel::<i32>::new(slot, slot_duration, now);
            let snapshot = wheel.dispatch(duration, data).await.unwrap();
            wheel.trigger(snapshot.id).await.unwrap();
            let timeout_ret = tokio::time::timeout(TOLERANCE, wheel.tick()).await.unwrap();
            assert_eq!(timeout_ret.len(), 1);
            assert_eq!(timeout_ret[0].id, snapshot.id);
            assert_eq!(timeout_ret[0].data, Some(data));
            assert_eq!(timeout_ret[0].start, snapshot.start);
            assert_ne!(timeout_ret[0].end, snapshot.end);
        });
    }

    #[test_case(
        60,
        Duration::from_secs(1),
        Duration::from_secs_f32(0.999),
        1,
        Duration::from_millis(700)
    )]
    #[test_case(
        30,
        Duration::from_secs(2),
        Duration::from_secs_f32(1.999),
        1,
        Duration::from_millis(1700)
    )]
    #[test_case(
        10,
        Duration::from_millis(100),
        Duration::from_millis(200),
        1,
        Duration::from_millis(150)
    )]
    #[test_case(
        5,
        Duration::from_millis(200),
        Duration::from_millis(380),
        1,
        Duration::from_millis(350)
    )]
    fn test_wheel_accelerate(
        slot: u32,
        slot_duration: Duration,
        duration: Duration,
        data: i32,
        accelerate: Duration,
    ) {
        let rt = build_runtime();
        rt.block_on(async {
            let now = Instant::now();
            let mut wheel = Wheel::<i32>::new(slot, slot_duration, now);
            let snapshot = wheel.dispatch(duration, data).await.unwrap();
            wheel.accelerate(snapshot.id, accelerate).await.unwrap();
            let timeout_ret = tokio::time::timeout(duration - accelerate, wheel.tick())
                .await
                .unwrap();
            assert_eq!(timeout_ret.len(), 1);
            assert_eq!(timeout_ret[0].id, snapshot.id);
            assert_eq!(timeout_ret[0].data, Some(data));
            assert_eq!(timeout_ret[0].start, snapshot.start);
            assert_ne!(timeout_ret[0].end, snapshot.end);
            assert_eq!(timeout_ret[0].end, snapshot.end - accelerate);
        });
    }
}
