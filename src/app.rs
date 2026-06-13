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
use crate::demo::DemoMode;
use crate::github::events::GitHubEvent;
use crate::github::polling::Poller;
use crate::notifications::NotificationManager;
use crate::render::pipeline::RenderPipeline;
use crate::render::theme::Theme;
use crate::render::timeline_view::TimelineView;
use crate::timeline::state::TimelineState;
use crate::ui::context_menu::{ContextAction, ContextMenu};
use crate::ui::settings_panel::SettingsPanel;

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
    #[allow(dead_code)]
    tokio_rt: tokio::runtime::Runtime,
    theme: Theme,
    demo: DemoMode,
    context_menu: ContextMenu,
    settings_panel: SettingsPanel,
    notification_manager: NotificationManager,
    pending_action: Option<ContextAction>,
    needs_render_prepare: bool,
    needs_redraw: bool,
    whoami_rx: mpsc::Receiver<String>,
}

impl App {
    pub fn new(config: Config) -> (Self, EventLoop<()>) {
        let (event_tx, event_rx) = mpsc::channel::<Vec<GitHubEvent>>(256);
        let (whoami_tx, whoami_rx) = mpsc::channel::<String>(1);

        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime");

        let poller = Poller::new(&config, event_tx);
        tokio_rt.spawn(async move {
            poller.run().await;
        });

        let whoami_token = config.github_token.clone();
        tokio_rt.spawn(async move {
            match crate::github::client::GithubClient::new(whoami_token) {
                Ok(client) => match client.whoami().await {
                    Ok(name) => {
                        let _ = whoami_tx.send(name).await;
                    }
                    Err(e) => {
                        tracing::warn!("whoami failed: {e}");
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to create client for whoami: {e}");
                }
            }
        });

        let timeline_state = TimelineState::new();
        let animation_manager = AnimationManager::new();
        let theme = Theme::from_config(&config.theme);
        let settings_panel = SettingsPanel::new(&config);

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
            demo: DemoMode::new(),
            context_menu: ContextMenu::new(),
            settings_panel,
            notification_manager: NotificationManager::new(false),
            pending_action: None,
            needs_render_prepare: true,
            needs_redraw: true,
            whoami_rx,
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
        let mut all_events = Vec::new();
        while let Ok(events) = self.event_rx.try_recv() {
            for event in events {
                debug!(
                    "New event: {} from {}",
                    event.event_type_label(),
                    event.actor
                );
                all_events.push(event.clone());
                self.timeline_state.update(vec![event]);
                self.needs_render_prepare = true;
                self.needs_redraw = true;
            }
        }

        while let Ok(name) = self.whoami_rx.try_recv() {
            info!("GitHub user: {name}");
            self.settings_panel.set_whoami(Some(name));
        }

        self.notification_manager.process_events(&all_events);

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

    fn process_pending_actions(&mut self) {
        if let Some(action) = self.pending_action.take() {
            match action {
                ContextAction::ToggleNotifications => {
                    self.notification_manager.toggle();
                    self.context_menu
                        .set_notifications_enabled(self.notification_manager.is_enabled());
                    self.needs_render_prepare = true;
                    self.needs_redraw = true;
                }
                ContextAction::ToggleDemo => {
                    if !self.demo.is_active() {
                        self.demo.start();
                    }
                    self.needs_redraw = true;
                }
                ContextAction::RefreshNow => {
                    info!("Manual refresh requested");
                    self.needs_redraw = true;
                }
                ContextAction::OpenSettings => {
                    self.settings_panel.toggle();
                    self.needs_render_prepare = true;
                    self.needs_redraw = true;
                }
                ContextAction::Exit => {}
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

    #[allow(dead_code)]
    pub fn start_demo(&mut self) {
        self.demo.start();
    }

    fn process_demo_events(&mut self) {
        if !self.demo.is_active() {
            return;
        }
        let events = self.demo.tick();
        if !events.is_empty() {
            self.timeline_state.update(events);
            self.needs_render_prepare = true;
            self.needs_redraw = true;
        }
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
                self.needs_redraw = true;
            }

            WindowEvent::CursorEntered { .. } => {
                self.is_hovered = true;
                self.update_global_opacity();
                self.needs_redraw = true;
            }

            WindowEvent::CursorLeft { .. } => {
                self.is_hovered = false;
                self.is_dragging = false;
                self.drag_start_pos = None;
                self.update_global_opacity();
                self.needs_redraw = true;
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.last_cursor_pos = (position.x, position.y);
                if self.is_dragging
                    && let Some(start) = self.drag_start_pos
                {
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
                    self.needs_redraw = true;
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Right && state == ElementState::Pressed {
                    self.context_menu
                        .show(self.last_cursor_pos.0 as f32, self.last_cursor_pos.1 as f32);
                    self.needs_render_prepare = true;
                    self.needs_redraw = true;
                }

                if button == MouseButton::Left {
                    match state {
                        ElementState::Pressed => {
                            self.is_dragging = true;
                            self.drag_start_pos = Some(self.last_cursor_pos);
                        }
                        ElementState::Released => {
                            let cx = self.last_cursor_pos.0 as f32;
                            let cy = self.last_cursor_pos.1 as f32;
                            let sw = self
                                .window
                                .as_ref()
                                .map(|w| w.inner_size().width as f32)
                                .unwrap_or(320.0);

                            if self.context_menu.is_visible() {
                                if let Some(action) = self.context_menu.hit_test(cx, cy) {
                                    match action {
                                        ContextAction::Exit => {
                                            self.context_menu.hide();
                                            event_loop.exit();
                                            return;
                                        }
                                        other => {
                                            self.pending_action = Some(other);
                                        }
                                    }
                                }
                                self.context_menu.hide();
                                self.needs_render_prepare = true;
                                self.needs_redraw = true;
                            } else if self.settings_panel.is_visible() {
                                if let Some(field_type) = self.settings_panel.hit_test(cx, cy, sw) {
                                    self.settings_panel
                                        .apply_field_click(&field_type, &mut self.config);
                                    self.needs_render_prepare = true;
                                    self.needs_redraw = true;
                                }
                                self.is_dragging = false;
                                self.drag_start_pos = None;
                            } else {
                                let start = self.drag_start_pos;
                                self.is_dragging = false;
                                if let Some(start_pos) = start {
                                    let dx = self.last_cursor_pos.0 - start_pos.0;
                                    let dy = self.last_cursor_pos.1 - start_pos.1;
                                    if dx.abs() < 5.0
                                        && dy.abs() < 5.0
                                        && let Some(ref view) = self.timeline_view
                                        && let Some(url) = view.hit_test(
                                            self.last_cursor_pos.0 as f32,
                                            self.last_cursor_pos.1 as f32,
                                        )
                                    {
                                        let _ = open::that(url);
                                    }
                                }
                                self.drag_start_pos = None;
                            }
                        }
                    }
                    self.needs_redraw = true;
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * 40.0,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
                self.timeline_state.scroll(scroll_amount);
                self.needs_render_prepare = true;
                self.needs_redraw = true;
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(self.last_frame_time);
                self.last_frame_time = now;

                self.animation_manager.tick(dt);
                self.needs_redraw = false;

                if self.needs_render_prepare
                    && let (Some(pipeline), Some(view)) =
                        (&mut self.render_pipeline, &mut self.timeline_view)
                {
                    let size = match self.window.as_ref().map(|w| w.inner_size()) {
                        Some(s) => s,
                        None => return,
                    };
                    view.prepare_render(
                        &pipeline.device,
                        &pipeline.queue,
                        self.timeline_state.get_entries(),
                        self.animation_manager.global_opacity(),
                        size.width as f32,
                        size.height as f32,
                        &self.context_menu,
                        &self.settings_panel,
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
        self.process_demo_events();
        if self.demo.is_active() {
            self.needs_redraw = true;
        }
        self.process_pending_actions();

        if self.needs_redraw || self.animation_manager.has_active_animations() {
            self.needs_redraw = false;
            if let Some(ref window) = self.window {
                window.request_redraw();
            }
        }
    }
}
