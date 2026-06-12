use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tracing::{debug, info};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::animation::manager::AnimationManager;
use crate::config::Config;
use crate::github::events::GitHubEvent;
use crate::github::polling::Poller;
use crate::render::pipeline::RenderPipeline;
use crate::render::theme::Theme;
use crate::render::timeline_view::TimelineView;
use crate::timeline::state::TimelineState;

const FADE_DURATION: Duration = Duration::from_millis(200);
const PULSE_DURATION: Duration = Duration::from_millis(600);

pub struct App {
    config: Config,
    timeline_state: TimelineState,
    animation_manager: AnimationManager,
    render_pipeline: Option<RenderPipeline>,
    timeline_view: Option<TimelineView>,
    event_rx: mpsc::Receiver<Vec<GitHubEvent>>,
    is_hovered: bool,
    is_dragging: bool,
    drag_start_pos: Option<(f64, f64)>,
    last_cursor_pos: (f64, f64),
    window_pos: Option<(i32, i32)>,
    last_frame_time: Instant,
    window: Option<Arc<Window>>,
    tokio_rt: tokio::runtime::Runtime,
    theme: Theme,
    needs_render_prepare: bool,
}

impl App {
    pub fn new(config: Config) -> (Self, EventLoop<()>) {
        let (event_tx, event_rx) = mpsc::channel::<Vec<GitHubEvent>>(256);

        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime");

        let poller = Poller::new(&config, event_tx);
        tokio_rt.spawn(async move {
            poller.run().await;
        });

        let timeline_state = TimelineState::new();
        let animation_manager = AnimationManager::new();
        let theme = Theme::from_config(&config.theme);

        let event_loop = EventLoop::new().expect("Failed to create event loop");

        let app = Self {
            config,
            timeline_state,
            animation_manager,
            render_pipeline: None,
            timeline_view: None,
            event_rx,
            is_hovered: false,
            is_dragging: false,
            drag_start_pos: None,
            last_cursor_pos: (0.0, 0.0),
            window_pos: None,
            last_frame_time: Instant::now(),
            window: None,
            tokio_rt,
            theme,
            needs_render_prepare: true,
        };

        (app, event_loop)
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        event_loop
            .run_app(&mut self)
            .expect("Event loop terminated with error");
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) {
        let mut attrs = WindowAttributes::default()
            .with_transparent(true)
            .with_decorations(false)
            .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.window.width,
                self.config.window.height,
            ));

        if let (Some(x), Some(y)) = (self.config.window.x, self.config.window.y) {
            attrs = attrs.with_position(LogicalPosition::new(x, y));
        }

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create window"),
        );

        let pipeline = pollster::block_on(RenderPipeline::new(window.clone()))
            .expect("Failed to create render pipeline");

        let timeline_view = TimelineView::new(
            &pipeline.device,
            &pipeline.queue,
            pipeline.surface_config.format,
            self.theme.clone(),
        );

        self.window = Some(window);
        self.render_pipeline = Some(pipeline);
        self.timeline_view = Some(timeline_view);
        self.window_pos = Some((
            self.config.window.x.unwrap_or(0),
            self.config.window.y.unwrap_or(0),
        ));
    }

    fn process_github_events(&mut self) {
        while let Ok(events) = self.event_rx.try_recv() {
            for event in events {
                debug!(
                    "New event: {} from {}",
                    event.event_type_label(),
                    event.actor
                );
                self.timeline_state.update(vec![event]);
                self.needs_render_prepare = true;
            }
        }

        while let Some(anim_event) = self.timeline_state.pop_animation() {
            match anim_event {
                crate::timeline::state::AnimationEvent::NewEntry(_) => {
                    self.animation_manager.add_fade_in(FADE_DURATION);
                }
                crate::timeline::state::AnimationEvent::UpdatedEntry(_) => {
                    self.animation_manager.add_pulse(PULSE_DURATION);
                }
            }
        }
    }

    fn update_global_opacity(&mut self) {
        let target = if self.is_hovered {
            self.config.window.hover_opacity
        } else {
            self.config.window.opacity
        };
        self.animation_manager.set_global_opacity(target);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.create_window(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting");
                event_loop.exit();
            }

            WindowEvent::Resized(new_size) => {
                if let Some(ref mut pipeline) = self.render_pipeline {
                    pipeline.resize(new_size.width, new_size.height);
                }
                self.needs_render_prepare = true;
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::CursorEntered { .. } => {
                self.is_hovered = true;
                self.update_global_opacity();
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::CursorLeft { .. } => {
                self.is_hovered = false;
                self.is_dragging = false;
                self.drag_start_pos = None;
                self.update_global_opacity();
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.last_cursor_pos = (position.x, position.y);
                if self.is_dragging
                    && let Some(start) = self.drag_start_pos {
                        let dx = (position.x - start.0) as i32;
                        let dy = (position.y - start.1) as i32;
                        if let Some(ref mut pos) = self.window_pos {
                            pos.0 += dx;
                            pos.1 += dy;
                            if let Some(ref window) = self.window {
                                window.set_outer_position(LogicalPosition::new(pos.0, pos.1));
                            }
                        }
                        self.drag_start_pos = Some((position.x, position.y));
                    }

                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    match state {
                        ElementState::Pressed => {
                            self.is_dragging = true;
                            self.drag_start_pos = Some(self.last_cursor_pos);
                        }
                        ElementState::Released => {
                            let start = self.drag_start_pos;
                            self.is_dragging = false;
                            if let Some(start_pos) = start {
                                let dx = self.last_cursor_pos.0 - start_pos.0;
                                let dy = self.last_cursor_pos.1 - start_pos.1;
                                if dx.abs() < 5.0 && dy.abs() < 5.0
                                    && let Some(ref view) = self.timeline_view
                                        && let Some(url) = view.hit_test(
                                            self.last_cursor_pos.0 as f32,
                                            self.last_cursor_pos.1 as f32,
                                        ) {
                                            let _ = open::that(url);
                                        }
                            }
                            self.drag_start_pos = None;
                        }
                    }
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * 40.0,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
                self.timeline_state.scroll(scroll_amount);
                self.needs_render_prepare = true;
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(self.last_frame_time);
                self.last_frame_time = now;

                self.animation_manager.tick(dt);

                if self.needs_render_prepare
                    && let (Some(pipeline), Some(view)) =
                        (&mut self.render_pipeline, &mut self.timeline_view)
                    {
                        let size = self.window.as_ref().map(|w| w.inner_size()).unwrap();
                        view.prepare_render(
                            &pipeline.device,
                            &pipeline.queue,
                            self.timeline_state.get_entries(),
                            self.animation_manager.global_opacity(),
                            size.width as f32,
                            size.height as f32,
                        );
                        self.needs_render_prepare = false;
                    }

                if let (Some(pipeline), Some(view)) =
                    (&mut self.render_pipeline, &mut self.timeline_view)
                {
                    pipeline.render(
                        view,
                        &self.timeline_state,
                        self.animation_manager.global_opacity(),
                        &self.animation_manager,
                    );
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.process_github_events();

        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}
