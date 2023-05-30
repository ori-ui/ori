use glyphon::{Buffer, FontSystem, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer};
use ori_graphics::{Rect, TextAlign, TextSection};
use wgpu::{Device, MultisampleState, Queue, RenderPass, TextureFormat};

fn create_area<'a>(
    section: &'a TextSection,
    buffer: &'a Buffer,
    font_system: &mut FontSystem,
    clip: Option<Rect>,
) -> TextArea<'a> {
    let rect = section.messure_buffer(font_system, buffer);

    let left = match section.h_align {
        TextAlign::Start => section.rect.left(),
        TextAlign::Center => section.rect.center().x - rect.width() / 2.0,
        TextAlign::End => section.rect.right() - rect.width(),
    };

    let top = match section.v_align {
        TextAlign::Start => section.rect.top(),
        TextAlign::Center => section.rect.center().y - rect.height() / 2.0,
        TextAlign::End => section.rect.bottom() - rect.height(),
    };

    let bounds = match clip {
        Some(clip) => {
            let rect = clip.intersect(section.rect);

            TextBounds {
                left: rect.left() as i32,
                top: rect.top() as i32,
                right: rect.right() as i32,
                bottom: rect.bottom() as i32,
            }
        }
        None => TextBounds {
            left: section.rect.left() as i32,
            top: section.rect.top() as i32,
            right: section.rect.right() as i32,
            bottom: section.rect.bottom() as i32,
        },
    };

    TextArea {
        buffer,
        left: left.round() as i32,
        top: top.round() as i32,
        bounds,
        default_color: glyphon::Color::rgba(
            (section.color.r * 255.0) as u8,
            (section.color.g * 255.0) as u8,
            (section.color.b * 255.0) as u8,
            (section.color.a * 255.0) as u8,
        ),
    }
}

pub struct TextPipeline {
    text_atlas: TextAtlas,
    swash_cache: SwashCache,
    layers: Vec<TextRenderer>,
}

impl TextPipeline {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        Self {
            text_atlas: TextAtlas::new(device, queue, format),
            swash_cache: SwashCache::new(),
            layers: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        font_system: &mut FontSystem,
        layer: usize,
        width: u32,
        height: u32,
        text: &[(&TextSection, Option<Rect>)],
    ) {
        if layer >= self.layers.len() {
            self.layers.resize_with(layer + 1, || {
                TextRenderer::new(
                    &mut self.text_atlas,
                    device,
                    MultisampleState {
                        count: 4,
                        ..Default::default()
                    },
                    None,
                )
            });
        }

        let layer = &mut self.layers[layer];

        // create buffers
        let buffers = text
            .iter()
            .map(|(section, _)| section.buffer(font_system))
            .collect::<Vec<_>>();

        // create areas
        let areas = text
            .iter()
            .zip(buffers.iter())
            .map(|((section, clip), buffer)| create_area(section, buffer, font_system, *clip))
            .collect::<Vec<_>>();

        // lock the font_system and prepare the layer
        layer
            .prepare(
                device,
                queue,
                font_system,
                &mut self.text_atlas,
                glyphon::Resolution { width, height },
                &areas,
                &mut self.swash_cache,
            )
            .unwrap();
    }

    pub fn render<'a>(&'a self, pass: &mut RenderPass<'a>, layer: usize) {
        let layer = &self.layers[layer];
        layer.render(&self.text_atlas, pass).unwrap();
    }
}
