use super::tween::{Easing, Tween};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpacityPhase {
    FadeIn,
    FadeOut,
    Pulse,
    Idle,
}

#[derive(Debug)]
pub struct OpacityAnimator {
    pub current_opacity: f32,
    pub target_opacity: f32,
    pub tween: Option<Tween>,
    pub phase: OpacityPhase,
}

impl OpacityAnimator {
    pub fn new(initial_opacity: f32) -> Self {
        Self {
            current_opacity: initial_opacity,
            target_opacity: initial_opacity,
            tween: None,
            phase: OpacityPhase::Idle,
        }
    }

    pub fn fade_in(&mut self, duration: Duration) {
        self.phase = OpacityPhase::FadeIn;
        self.target_opacity = 1.0;
        self.tween = Some(Tween::new(
            self.current_opacity,
            1.0,
            duration,
            Easing::EaseOutCubic,
        ));
    }

    pub fn fade_out(&mut self, duration: Duration, target: f32) {
        self.phase = OpacityPhase::FadeOut;
        self.target_opacity = target;
        self.tween = Some(Tween::new(
            self.current_opacity,
            target,
            duration,
            Easing::EaseInCubic,
        ));
    }

    pub fn pulse(&mut self, duration: Duration) {
        self.phase = OpacityPhase::Pulse;
        self.target_opacity = self.current_opacity;
        let half = duration / 2;
        self.tween = Some(Tween::new(
            self.current_opacity,
            1.0,
            half,
            Easing::EaseInOut,
        ));
    }

    pub fn tick(&mut self, dt: Duration) -> f32 {
        if let Some(ref mut tween) = self.tween {
            self.current_opacity = tween.tick(dt);
            if tween.is_complete() {
                match self.phase {
                    OpacityPhase::FadeIn => {
                        self.tween = None;
                        self.phase = OpacityPhase::Idle;
                    }
                    OpacityPhase::FadeOut => {
                        self.tween = None;
                        self.phase = OpacityPhase::Idle;
                    }
                    OpacityPhase::Pulse => {
                        let duration = tween.duration;
                        let half = duration / 2;
                        self.tween = Some(Tween::new(
                            1.0,
                            self.target_opacity,
                            half,
                            Easing::EaseInOut,
                        ));
                        self.phase = OpacityPhase::Idle;
                    }
                    OpacityPhase::Idle => {
                        self.tween = None;
                    }
                }
            }
        }
        self.current_opacity
    }

    pub fn is_animating(&self) -> bool {
        self.tween.is_some()
    }

    pub fn opacity(&self) -> f32 {
        self.current_opacity
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
    fn new_starts_at_given_opacity() {
        let a = OpacityAnimator::new(0.5);
        assert!(approx_eq(a.opacity(), 0.5));
        assert!(approx_eq(a.current_opacity, 0.5));
    }

    #[test]
    fn new_starts_idle() {
        let a = OpacityAnimator::new(0.0);
        assert_eq!(a.phase, OpacityPhase::Idle);
        assert!(!a.is_animating());
    }

    #[test]
    fn fade_in_sets_phase() {
        let mut a = OpacityAnimator::new(0.0);
        a.fade_in(Duration::from_millis(500));
        assert_eq!(a.phase, OpacityPhase::FadeIn);
        assert!(a.is_animating());
    }

    #[test]
    fn fade_in_sets_target_to_one() {
        let mut a = OpacityAnimator::new(0.0);
        a.fade_in(Duration::from_millis(500));
        assert!(approx_eq(a.target_opacity, 1.0));
    }

    #[test]
    fn fade_out_sets_phase() {
        let mut a = OpacityAnimator::new(1.0);
        a.fade_out(Duration::from_millis(500), 0.0);
        assert_eq!(a.phase, OpacityPhase::FadeOut);
        assert!(a.is_animating());
    }

    #[test]
    fn fade_out_sets_target() {
        let mut a = OpacityAnimator::new(1.0);
        a.fade_out(Duration::from_millis(500), 0.2);
        assert!(approx_eq(a.target_opacity, 0.2));
    }

    #[test]
    fn pulse_sets_phase() {
        let mut a = OpacityAnimator::new(0.5);
        a.pulse(Duration::from_millis(400));
        assert_eq!(a.phase, OpacityPhase::Pulse);
        assert!(a.is_animating());
    }

    #[test]
    fn is_animating_false_when_idle() {
        let a = OpacityAnimator::new(1.0);
        assert!(!a.is_animating());
    }

    #[test]
    fn tick_changes_opacity_during_fade_in() {
        let mut a = OpacityAnimator::new(0.0);
        a.fade_in(Duration::from_millis(100));
        a.tick(Duration::from_millis(50));
        assert!(a.opacity() > 0.0);
        assert!(a.opacity() < 1.0);
    }

    #[test]
    fn tick_completes_fade_in() {
        let mut a = OpacityAnimator::new(0.0);
        a.fade_in(Duration::from_millis(100));
        a.tick(Duration::from_millis(200));
        assert!(approx_eq(a.opacity(), 1.0));
        assert_eq!(a.phase, OpacityPhase::Idle);
        assert!(!a.is_animating());
    }

    #[test]
    fn tick_completes_fade_out() {
        let mut a = OpacityAnimator::new(1.0);
        a.fade_out(Duration::from_millis(100), 0.0);
        a.tick(Duration::from_millis(200));
        assert!(approx_eq(a.opacity(), 0.0));
        assert_eq!(a.phase, OpacityPhase::Idle);
        assert!(!a.is_animating());
    }

    #[test]
    fn opacity_returns_current_value() {
        let mut a = OpacityAnimator::new(0.3);
        assert!(approx_eq(a.opacity(), 0.3));
        a.fade_in(Duration::from_millis(100));
        a.tick(Duration::from_millis(100));
        assert!(approx_eq(a.opacity(), 1.0));
    }
}
