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
