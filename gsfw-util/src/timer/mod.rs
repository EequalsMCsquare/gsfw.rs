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

    async fn test_timer_impl(now: Instant, end: Instant, data: i32) {
        let ret = Timer::new(now, end, data).await;
        assert_eq!(ret.unwrap(), data);
    }

    async fn test_wheel_dispatch_impl_1(
        now: Instant,
        slot: u32,
        slot_duration: Duration,
        dispatch_duration: Duration,
        data: i32,
    ) {
        let mut wheel = Wheel::<i32>::new(slot, slot_duration, now);
        let snapshot = wheel.dispatch(dispatch_duration, data).await.unwrap();
        let recv_data = wheel.tick().await;
        assert_eq!(recv_data.len(), 1);
        assert_eq!(recv_data[0].id, snapshot.id);
        assert_eq!(recv_data[0].start, snapshot.start);
        assert_eq!(recv_data[0].end, snapshot.end);
        assert_eq!(recv_data[0].data, Some(data));
    }

    async fn test_wheel_dispatch_until_impl_1(
        now: Instant,
        slot: u32,
        slot_duration: Duration,
        dispatch_until: Instant,
        data: i32,
    ) {
        let mut wheel = Wheel::<i32>::new(slot, slot_duration, now);
        let snapshot = wheel.dispatch_until(dispatch_until, data).await.unwrap();
        let recv_data = wheel.tick().await;
        assert_eq!(recv_data.len(), 1);
        assert_eq!(recv_data[0].id, snapshot.id);
        assert_eq!(recv_data[0].start, snapshot.start);
        assert_eq!(recv_data[0].end, snapshot.end);
        assert_eq!(recv_data[0].data, Some(data));
    }

    #[tokio::test]
    async fn test_timer_1() {
        let now = std::time::Instant::now();
        test_timer_impl(now, now + Duration::from_millis(100), 1).await;
        test_timer_impl(now, now + Duration::from_millis(200), 1).await;
        test_timer_impl(now, now + Duration::from_millis(300), 1).await;
        test_timer_impl(now, now + Duration::from_millis(400), 1).await;
        test_timer_impl(now, now + Duration::from_millis(500), 1).await;
    }

    #[tokio::test]
    async fn test_timer_2() {
        let now = std::time::Instant::now();
        test_timer_impl(now, now + Duration::from_millis(100), 1).await;
        let now = std::time::Instant::now();
        test_timer_impl(now, now + Duration::from_millis(200), 1).await;
        let now = std::time::Instant::now();
        test_timer_impl(now, now + Duration::from_millis(300), 1).await;
        let now = std::time::Instant::now();
        test_timer_impl(now, now + Duration::from_millis(400), 1).await;
        let now = std::time::Instant::now();
        test_timer_impl(now, now + Duration::from_millis(500), 1).await;
    }

    #[tokio::test]
    async fn test_wheel_dispatch_1() {
        let now = Instant::now();
        test_wheel_dispatch_impl_1(now, 60, Duration::from_secs(1), Duration::from_secs(1), 1)
            .await;
        test_wheel_dispatch_impl_1(now, 30, Duration::from_secs(2), Duration::from_secs(1), 1)
            .await;
        test_wheel_dispatch_impl_1(now, 20, Duration::from_secs(3), Duration::from_secs(1), 1)
            .await;
        test_wheel_dispatch_impl_1(now, 15, Duration::from_secs(4), Duration::from_secs(1), 1)
            .await;
        test_wheel_dispatch_impl_1(now, 12, Duration::from_secs(5), Duration::from_secs(1), 1)
            .await;
        test_wheel_dispatch_impl_1(now, 10, Duration::from_secs(6), Duration::from_secs(1), 1)
            .await;
    }

    #[tokio::test]
    async fn test_wheel_dispatch_until_1() {
        let now = Instant::now();
        test_wheel_dispatch_until_impl_1(now, 60, Duration::from_secs(1), now + Duration::from_secs(1), 1).await;
        test_wheel_dispatch_until_impl_1(now, 30, Duration::from_secs(2), now + Duration::from_secs(1), 1).await;
        test_wheel_dispatch_until_impl_1(now, 20, Duration::from_secs(3), now + Duration::from_secs(1), 1).await;
        test_wheel_dispatch_until_impl_1(now, 15, Duration::from_secs(4), now + Duration::from_secs(1), 1).await;
        test_wheel_dispatch_until_impl_1(now, 12, Duration::from_secs(5), now + Duration::from_secs(1), 1).await;
        test_wheel_dispatch_until_impl_1(now, 10, Duration::from_secs(6), now + Duration::from_secs(1), 1).await;
    }
}
