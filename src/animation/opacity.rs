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
