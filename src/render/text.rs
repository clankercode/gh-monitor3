use glyphon::{
    Attrs, Buffer, Cache, Color, FontSystem, Metrics, Shaping, SwashCache, TextAtlas,
    TextRenderer as GlyphonRenderer, Viewport,
};

pub struct TextSegment {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
    pub color: [f32; 4],
    pub max_width: Option<f32>,
}

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    #[allow(dead_code)]
    cache: Cache,
    atlas: TextAtlas,
    text_renderer: GlyphonRenderer,
    viewport: Viewport,
    buffers: Vec<Buffer>,
}

impl TextRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let mut font_system = FontSystem::new();

        let emoji_paths = [
            "/usr/share/fonts/noto/NotoColorEmoji.ttf",
            "/usr/share/fonts/google-noto-emoji/NotoColorEmoji.ttf",
            "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
            "/usr/share/fonts/emoji/NotoColorEmoji.ttf",
            "/System/Library/Fonts/Apple Color Emoji.ttc",
            "C:\\Windows\\Fonts\\seguiemj.ttf",
        ];
        for path in &emoji_paths {
            if std::path::Path::new(path).exists()
                && let Ok(data) = std::fs::read(path)
            {
                font_system.db_mut().load_font_data(data);
            }
        }

        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let text_renderer =
            GlyphonRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let viewport = Viewport::new(device, &cache);

        Self {
            font_system,
            swash_cache,
            cache,
            atlas,
            text_renderer,
            viewport,
            buffers: Vec::new(),
        }
    }

    pub fn prepare_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texts: &[TextSegment],
        width: u32,
        height: u32,
    ) {
        self.buffers.clear();

        self.viewport
            .update(queue, glyphon::Resolution { width, height });

        for segment in texts {
            let mut buffer = Buffer::new(
                &mut self.font_system,
                Metrics::new(segment.font_size, segment.font_size * 1.4),
            );
            let max_w = segment.max_width.unwrap_or(f32::MAX);
            buffer.set_size(&mut self.font_system, Some(max_w), None);
            buffer.set_text(
                &mut self.font_system,
                &segment.text,
                &Attrs::new(),
                Shaping::Basic,
                None,
            );
            buffer.shape_until_scroll(&mut self.font_system, false);
            self.buffers.push(buffer);
        }

        let text_areas: Vec<_> = self
            .buffers
            .iter()
            .zip(texts.iter())
            .map(|(buffer, segment)| {
                let color = segment.color;
                glyphon::TextArea {
                    buffer,
                    left: segment.x,
                    top: segment.y,
                    scale: 1.0,
                    bounds: glyphon::TextBounds {
                        left: segment.x as i32,
                        top: segment.y as i32,
                        right: (segment.x + segment.max_width.unwrap_or(width as f32)) as i32,
                        bottom: (segment.y + segment.font_size * 3.0) as i32,
                    },
                    default_color: Color::rgba(
                        (color[0] * 255.0) as u8,
                        (color[1] * 255.0) as u8,
                        (color[2] * 255.0) as u8,
                        (color[3] * 255.0) as u8,
                    ),
                    custom_glyphs: &[],
                }
            })
            .collect();

        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .expect("failed to prepare text");
    }

    pub fn render<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        self.text_renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .expect("failed to render text");
    }

    #[allow(dead_code)]
    pub fn trim(&mut self) {
        self.atlas.trim();
    }
}
