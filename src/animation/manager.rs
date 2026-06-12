use super::opacity::OpacityAnimator;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub type AnimationId = u64;

pub struct ManagedAnimation {
    pub id: AnimationId,
    pub opacity_animator: OpacityAnimator,
    pub position_offset: Option<(f32, f32)>,
    pub created_at: Instant,
}

pub struct AnimationManager {
    animations: HashMap<AnimationId, ManagedAnimation>,
    next_id: AnimationId,
    global_opacity: f32,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            next_id: 1,
            global_opacity: 1.0,
        }
    }

    pub fn add_fade_in(&mut self, duration: Duration) -> AnimationId {
        let id = self.next_id;
        self.next_id += 1;
        let mut animator = OpacityAnimator::new(0.0);
        animator.fade_in(duration);
        self.animations.insert(
            id,
            ManagedAnimation {
                id,
                opacity_animator: animator,
                position_offset: None,
                created_at: Instant::now(),
            },
        );
        id
    }

    pub fn add_pulse(&mut self, duration: Duration) -> AnimationId {
        let id = self.next_id;
        self.next_id += 1;
        let mut animator = OpacityAnimator::new(0.0);
        animator.pulse(duration);
        self.animations.insert(
            id,
            ManagedAnimation {
                id,
                opacity_animator: animator,
                position_offset: None,
                created_at: Instant::now(),
            },
        );
        id
    }

    pub fn remove(&mut self, id: AnimationId) {
        self.animations.remove(&id);
    }

    pub fn tick(&mut self, dt: Duration) {
        let mut completed = Vec::new();
        for (id, animation) in &mut self.animations {
            animation.opacity_animator.tick(dt);
            if !animation.opacity_animator.is_animating() {
                completed.push(*id);
            }
        }
        for id in completed {
            self.animations.remove(&id);
        }
    }

    pub fn get_opacity(&self, id: AnimationId) -> Option<f32> {
        self.animations
            .get(&id)
            .map(|a| a.opacity_animator.opacity())
    }

    pub fn set_global_opacity(&mut self, opacity: f32) {
        self.global_opacity = opacity.clamp(0.0, 1.0);
    }

    pub fn global_opacity(&self) -> f32 {
        self.global_opacity
    }

    pub fn has_active_animations(&self) -> bool {
        !self.animations.is_empty()
    }
}
