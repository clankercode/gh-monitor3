use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInBack,
    EaseOutBack,
}

impl Easing {
    pub fn ease(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t).powi(2),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::EaseInCubic => t.powi(3),
            Easing::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t.powi(3)
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::EaseInBack => {
                let s = 1.70158;
                (s + 1.0) * t.powi(3) - s * t.powi(2)
            }
            Easing::EaseOutBack => {
                let s = 1.70158;
                let t = t - 1.0;
                (s + 1.0) * t.powi(3) + s * t.powi(2) + 1.0
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tween {
    pub start_value: f32,
    pub end_value: f32,
    pub duration: Duration,
    pub elapsed: Duration,
    pub easing: Easing,
}

impl Tween {
    pub fn new(start_value: f32, end_value: f32, duration: Duration, easing: Easing) -> Self {
        Self {
            start_value,
            end_value,
            duration,
            elapsed: Duration::ZERO,
            easing,
        }
    }

    pub fn tick(&mut self, dt: Duration) -> f32 {
        self.elapsed += dt;
        self.value()
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
    }

    pub fn value(&self) -> f32 {
        if self.duration.is_zero() {
            return self.end_value;
        }
        let t = if self.is_complete() {
            1.0
        } else {
            self.elapsed.as_secs_f32() / self.duration.as_secs_f32()
        };
        let eased = self.easing.ease(t);
        self.start_value + (self.end_value - self.start_value) * eased
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-6;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn linear_returns_input() {
        assert!(approx_eq(Easing::Linear.ease(0.0), 0.0));
        assert!(approx_eq(Easing::Linear.ease(0.5), 0.5));
        assert!(approx_eq(Easing::Linear.ease(1.0), 1.0));
    }

    #[test]
    fn ease_in_at_boundaries() {
        assert!(approx_eq(Easing::EaseIn.ease(0.0), 0.0));
        assert!(approx_eq(Easing::EaseIn.ease(1.0), 1.0));
    }

    #[test]
    fn ease_in_at_midpoint() {
        assert!(approx_eq(Easing::EaseIn.ease(0.5), 0.25));
    }

    #[test]
    fn ease_out_at_boundaries() {
        assert!(approx_eq(Easing::EaseOut.ease(0.0), 0.0));
        assert!(approx_eq(Easing::EaseOut.ease(1.0), 1.0));
    }

    #[test]
    fn ease_out_at_midpoint() {
        assert!(approx_eq(Easing::EaseOut.ease(0.5), 0.75));
    }

    #[test]
    fn ease_in_out_at_boundaries() {
        assert!(approx_eq(Easing::EaseInOut.ease(0.0), 0.0));
        assert!(approx_eq(Easing::EaseInOut.ease(1.0), 1.0));
    }

    #[test]
    fn ease_in_out_at_midpoint() {
        assert!(approx_eq(Easing::EaseInOut.ease(0.5), 0.5));
    }

    #[test]
    fn tween_new_initial_state() {
        let t = Tween::new(0.0, 100.0, Duration::from_secs(1), Easing::Linear);
        assert_eq!(t.start_value, 0.0);
        assert_eq!(t.end_value, 100.0);
        assert_eq!(t.duration, Duration::from_secs(1));
        assert_eq!(t.elapsed, Duration::ZERO);
        assert_eq!(t.easing, Easing::Linear);
    }

    #[test]
    fn tween_tick_advances_time() {
        let mut t = Tween::new(0.0, 10.0, Duration::from_secs(1), Easing::Linear);
        t.tick(Duration::from_millis(500));
        assert_eq!(t.elapsed, Duration::from_millis(500));
    }

    #[test]
    fn tween_tick_returns_interpolated_value() {
        let mut t = Tween::new(0.0, 10.0, Duration::from_secs(1), Easing::Linear);
        let val = t.tick(Duration::from_millis(500));
        assert!(approx_eq(val, 5.0));
    }

    #[test]
    fn tween_is_complete_false_before_duration() {
        let mut t = Tween::new(0.0, 1.0, Duration::from_secs(1), Easing::Linear);
        t.tick(Duration::from_millis(500));
        assert!(!t.is_complete());
    }

    #[test]
    fn tween_is_complete_true_after_duration() {
        let mut t = Tween::new(0.0, 1.0, Duration::from_secs(1), Easing::Linear);
        t.tick(Duration::from_secs(2));
        assert!(t.is_complete());
    }

    #[test]
    fn tween_is_complete_true_at_duration() {
        let mut t = Tween::new(0.0, 1.0, Duration::from_secs(1), Easing::Linear);
        t.tick(Duration::from_secs(1));
        assert!(t.is_complete());
    }

    #[test]
    fn tween_reset() {
        let mut t = Tween::new(0.0, 1.0, Duration::from_secs(1), Easing::Linear);
        t.tick(Duration::from_millis(500));
        t.reset();
        assert_eq!(t.elapsed, Duration::ZERO);
        assert!(!t.is_complete());
    }

    #[test]
    fn tween_value_at_start() {
        let t = Tween::new(10.0, 20.0, Duration::from_secs(1), Easing::Linear);
        assert!(approx_eq(t.value(), 10.0));
    }

    #[test]
    fn tween_value_at_end() {
        let mut t = Tween::new(10.0, 20.0, Duration::from_secs(1), Easing::Linear);
        t.tick(Duration::from_secs(2));
        assert!(approx_eq(t.value(), 20.0));
    }

    #[test]
    fn tween_value_zero_duration() {
        let t = Tween::new(5.0, 15.0, Duration::ZERO, Easing::Linear);
        assert!(approx_eq(t.value(), 15.0));
    }

    #[test]
    fn tween_clamps_past_end() {
        let mut t = Tween::new(0.0, 100.0, Duration::from_secs(1), Easing::Linear);
        let val = t.tick(Duration::from_secs(5));
        assert!(approx_eq(val, 100.0));
    }
}
