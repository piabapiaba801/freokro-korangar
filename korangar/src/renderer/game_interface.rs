use std::cell::{Ref, RefCell};
use std::sync::Arc;

use cgmath::Point3;
use cgmath::{EuclideanSpace, Vector2};

use crate::graphics::{Color, RectangleInstruction, ScreenClip, ScreenPosition, ScreenSize, Texture};
use crate::loaders::{FontLoader, FontSize, GlyphInstruction, Scaling};
#[cfg(feature = "debug")]
use crate::loaders::{ImageType, TextureLoader};
#[cfg(feature = "debug")]
use crate::renderer::MarkerRenderer;
use crate::renderer::SpriteRenderer;
use crate::world::Camera;
#[cfg(feature = "debug")]
use crate::world::MarkerIdentifier;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[allow(unused)]
pub enum AlignHorizontal {
    Left,
    Center,
}

/// Renders the in-game interface (like the FPS counter, the mouse pointer or
/// the health bars).
pub struct GameInterfaceRenderer {
    instructions: RefCell<Vec<RectangleInstruction>>,
    glyphs: RefCell<Vec<GlyphInstruction>>,
    font_loader: Arc<FontLoader>,
    window_size: ScreenSize,
    scaling: Scaling,
    highlight_color: Color,
    #[cfg(feature = "debug")]
    object_marker_texture: Arc<Texture>,
    #[cfg(feature = "debug")]
    light_source_marker_texture: Arc<Texture>,
    #[cfg(feature = "debug")]
    sound_source_marker_texture: Arc<Texture>,
    #[cfg(feature = "debug")]
    effect_source_marker_texture: Arc<Texture>,
    #[cfg(feature = "debug")]
    entity_marker_texture: Arc<Texture>,
    #[cfg(feature = "debug")]
    shadow_marker_texture: Arc<Texture>,
}

impl SpriteRenderer for GameInterfaceRenderer {
    fn render_sprite(
        &self,
        texture: Arc<Texture>,
        position: ScreenPosition,
        size: ScreenSize,
        _screen_clip: ScreenClip,
        color: Color,
        smooth: bool,
    ) {
        self.render_indexed(texture, position, size, color, 1, 0, smooth);
    }

    fn render_sdf(
        &self,
        texture: Arc<Texture>,
        screen_position: ScreenPosition,
        screen_size: ScreenSize,
        _screen_clip: ScreenClip,
        color: Color,
    ) {
        let screen_position = ScreenPosition {
            left: screen_position.left / self.window_size.width,
            top: screen_position.top / self.window_size.height,
        };

        let screen_size = ScreenSize {
            width: screen_size.width / self.window_size.width,
            height: screen_size.height / self.window_size.height,
        };

        let texture_position = Vector2::new(0.0, 0.0);
        let texture_size = Vector2::new(1.0, 1.0);

        self.instructions.borrow_mut().push(RectangleInstruction::Sdf {
            screen_position,
            screen_size,
            color,
            texture_position,
            texture_size,
            texture,
        });
    }
}

impl GameInterfaceRenderer {
    pub fn new(
        window_size: ScreenSize,
        scaling: Scaling,
        font_loader: Arc<FontLoader>,
        #[cfg(feature = "debug")] texture_loader: &TextureLoader,
    ) -> Self {
        let instructions = RefCell::new(Vec::new());
        let glyphs = RefCell::new(Vec::new());

        #[cfg(feature = "debug")]
        let object_marker_texture = texture_loader.get_or_load("marker_object.png", ImageType::Sdf).unwrap();
        #[cfg(feature = "debug")]
        let light_source_marker_texture = texture_loader.get_or_load("marker_light.png", ImageType::Sdf).unwrap();
        #[cfg(feature = "debug")]
        let sound_source_marker_texture = texture_loader.get_or_load("marker_sound.png", ImageType::Sdf).unwrap();
        #[cfg(feature = "debug")]
        let effect_source_marker_texture = texture_loader.get_or_load("marker_effect.png", ImageType::Sdf).unwrap();
        #[cfg(feature = "debug")]
        let entity_marker_texture = texture_loader.get_or_load("marker_entity.png", ImageType::Sdf).unwrap();
        #[cfg(feature = "debug")]
        let shadow_marker_texture = texture_loader.get_or_load("marker_shadow.png", ImageType::Sdf).unwrap();

        Self {
            instructions,
            glyphs,
            font_loader,
            window_size,
            scaling,
            highlight_color: Color::rgb_u8(255, 200, 150),
            #[cfg(feature = "debug")]
            object_marker_texture,
            #[cfg(feature = "debug")]
            light_source_marker_texture,
            #[cfg(feature = "debug")]
            sound_source_marker_texture,
            #[cfg(feature = "debug")]
            effect_source_marker_texture,
            #[cfg(feature = "debug")]
            entity_marker_texture,
            #[cfg(feature = "debug")]
            shadow_marker_texture,
        }
    }

    pub fn from_renderer(other: &Self) -> Self {
        Self {
            instructions: RefCell::new(Vec::default()),
            glyphs: RefCell::new(Vec::default()),
            font_loader: Arc::clone(&other.font_loader),
            window_size: other.window_size,
            scaling: other.scaling,
            highlight_color: other.highlight_color,
            #[cfg(feature = "debug")]
            object_marker_texture: other.object_marker_texture.clone(),
            #[cfg(feature = "debug")]
            light_source_marker_texture: other.light_source_marker_texture.clone(),
            #[cfg(feature = "debug")]
            sound_source_marker_texture: other.sound_source_marker_texture.clone(),
            #[cfg(feature = "debug")]
            effect_source_marker_texture: other.effect_source_marker_texture.clone(),
            #[cfg(feature = "debug")]
            entity_marker_texture: other.entity_marker_texture.clone(),
            #[cfg(feature = "debug")]
            shadow_marker_texture: other.shadow_marker_texture.clone(),
        }
    }

    pub fn clear(&self) {
        self.instructions.borrow_mut().clear();
    }

    pub fn get_instructions(&self) -> Ref<'_, Vec<RectangleInstruction>> {
        self.instructions.borrow()
    }

    pub fn update_window_size(&mut self, window_size: ScreenSize) {
        self.window_size = window_size;
    }

    pub fn update_scaling(&mut self, scaling: Scaling) {
        self.scaling = scaling;
    }

    pub fn render_text(
        &self,
        text: &str,
        text_position: ScreenPosition,
        color: Color,
        font_size: FontSize,
        align_horizontal: AlignHorizontal,
    ) {
        let font_size = FontSize(font_size.0 * self.scaling.get_factor());

        let mut glyphs = self.glyphs.borrow_mut();

        let size = self
            .font_loader
            .layout_text(text, color, self.highlight_color, font_size, 1.0, None, Some(&mut glyphs));

        let horizontal_offset = match align_horizontal {
            AlignHorizontal::Left => 0.0,
            AlignHorizontal::Center => -size.x / 2.0,
        };

        let mut instructions = self.instructions.borrow_mut();

        glyphs.drain(..).for_each(
            |GlyphInstruction {
                 position,
                 texture_coordinate,
                 color,
             }| {
                let screen_position = ScreenPosition {
                    left: text_position.left + position.min.x + horizontal_offset,
                    top: text_position.top + position.min.y,
                } / self.window_size;

                let screen_size = ScreenSize {
                    width: position.width(),
                    height: position.height(),
                } / self.window_size;

                let texture_position = texture_coordinate.min.to_vec();
                let texture_size = texture_coordinate.max - texture_coordinate.min;

                instructions.push(RectangleInstruction::Text {
                    screen_position,
                    screen_size,
                    color,
                    texture_position,
                    texture_size,
                });
            },
        );
    }

    /// Ragnarok-style speech bubble anchored to an entity in the world.
    pub fn render_speech_bubble(&self, camera: &dyn Camera, world_position: Point3<f32>, text: &str, text_color: Color) {
        let clip_position = camera.view_projection_matrix() * world_position.to_homogeneous();
        if clip_position.w <= 0.1 {
            return;
        }
        let screen = camera.clip_to_screen_space(clip_position);
        if !screen.x.is_finite() || !screen.y.is_finite() || screen.x < -0.05 || screen.x > 1.05 || screen.y < -0.05 || screen.y > 1.05 {
            return;
        }

        let scale = self.scaling.get_factor();
        let padding = 8.0 * scale;
        let max_text_width = (self.window_size.width - 32.0 * scale).min(300.0 * scale).max(64.0);
        let mut glyphs = self.glyphs.borrow_mut();
        glyphs.clear();
        let text_size = self.font_loader.layout_text(
            text,
            text_color,
            text_color,
            FontSize(14.0 * scale),
            1.0,
            Some(max_text_width),
            Some(&mut glyphs),
        );
        let bubble_width = text_size.x + padding * 2.0;
        let bubble_height = text_size.y + padding * 2.0;
        let anchor_x = screen.x * self.window_size.width;
        let anchor_y = screen.y * self.window_size.height;
        let left = (anchor_x - bubble_width / 2.0).clamp(8.0, (self.window_size.width - bubble_width - 8.0).max(8.0));
        let top = (anchor_y - bubble_height - 10.0 * scale).clamp(8.0, (self.window_size.height - bubble_height - 8.0).max(8.0));
        let border = 2.0 * scale;

        self.render_rectangle(
            ScreenPosition { left: left + 3.0 * scale, top: top + 3.0 * scale },
            ScreenSize { width: bubble_width, height: bubble_height },
            Color::rgba_u8(12, 19, 28, 90),
        );
        self.render_rectangle(
            ScreenPosition { left, top },
            ScreenSize { width: bubble_width, height: bubble_height },
            Color::rgb_u8(59, 52, 46),
        );
        self.render_rectangle(
            ScreenPosition { left: left + border, top: top + border },
            ScreenSize { width: bubble_width - border * 2.0, height: bubble_height - border * 2.0 },
            Color::rgb_u8(255, 251, 234),
        );
        let tail_left = anchor_x.clamp(left + 10.0 * scale, left + bubble_width - 10.0 * scale) - 5.0 * scale;
        self.render_rectangle(
            ScreenPosition { left: tail_left, top: top + bubble_height },
            ScreenSize { width: 10.0 * scale, height: 5.0 * scale },
            Color::rgb_u8(59, 52, 46),
        );
        self.render_rectangle(
            ScreenPosition { left: tail_left + 2.0 * scale, top: top + bubble_height },
            ScreenSize { width: 6.0 * scale, height: 3.0 * scale },
            Color::rgb_u8(255, 251, 234),
        );

        let mut instructions = self.instructions.borrow_mut();
        for GlyphInstruction { position, texture_coordinate, color } in glyphs.drain(..) {
            instructions.push(RectangleInstruction::Text {
                screen_position: ScreenPosition {
                    left: left + padding + position.min.x,
                    top: top + padding + position.min.y,
                } / self.window_size,
                screen_size: ScreenSize { width: position.width(), height: position.height() } / self.window_size,
                color,
                texture_position: texture_coordinate.min.to_vec(),
                texture_size: texture_coordinate.max - texture_coordinate.min,
            });
        }
    }

    pub fn render_hover_text(&self, text: &str, scaling: Scaling, mouse_position: ScreenPosition) {
        let offset = ScreenPosition {
            left: 15.0 * scaling.get_factor(),
            top: 15.0 * scaling.get_factor(),
        };

        self.render_text(
            text,
            mouse_position + offset,
            Color::WHITE,
            FontSize(16.0),
            AlignHorizontal::Center,
        );
    }

    pub fn render_damage_text(&self, text: &str, position: ScreenPosition, color: Color, font_size: FontSize) {
        self.render_text(text, position, color, font_size, AlignHorizontal::Center);
    }

    pub fn render_bar(&self, position: ScreenPosition, size: ScreenSize, color: Color, maximum: f32, current: f32) {
        let bar_offset = ScreenSize::only_width(size.width / 2.0);
        let bar_size = ScreenSize {
            width: (size.width / maximum) * current,
            height: size.height,
        };

        self.render_rectangle(position - bar_offset, bar_size, color);
    }

    /// Translucent progression bars fixed to the top and bottom edges.
    pub fn render_experience_bars(&self, base: u64, next_base: u64, job: u64, next_job: u64) {
        let width = self.window_size.width;
        let height = self.window_size.height;
        if width <= 0.0 || height < 16.0 {
            return;
        }

        let bar_height = 14.0;
        let background = Color::rgba_u8(17, 24, 20, 210);
        let fill = Color::rgba_u8(93, 177, 71, 220);
        for (top, current, next, label) in [
            (0.0, base, next_base, "XP Base"),
            (height - bar_height, job, next_job, "XP Job"),
        ] {
            let position = ScreenPosition { left: 0.0, top };
            self.render_rectangle(position, ScreenSize { width, height: bar_height }, background);
            self.render_rectangle(
                ScreenPosition { left: 0.0, top: top + bar_height - 2.0 },
                ScreenSize { width, height: 2.0 },
                fill,
            );
            if next > 0 {
                let progress = (current as f64 / next as f64).clamp(0.0, 1.0) as f32;
                if progress > 0.0 {
                    self.render_rectangle(position, ScreenSize { width: width * progress, height: bar_height - 2.0 }, fill);
                }
            }
            for segment in 1..10 {
                self.render_rectangle(
                    ScreenPosition { left: width * segment as f32 / 10.0, top: top + 1.0 },
                    ScreenSize { width: 2.0, height: bar_height - 3.0 },
                    Color::rgba_u8(8, 16, 11, 215),
                );
            }
            self.render_text(
                label,
                ScreenPosition { left: 6.0, top: top + 1.0 },
                Color::rgba_u8(245, 248, 252, 235),
                FontSize(11.0),
                AlignHorizontal::Left,
            );
        }
    }

    pub fn render_rectangle(&self, position: ScreenPosition, size: ScreenSize, color: Color) {
        let screen_position = position / self.window_size;
        let screen_size = size / self.window_size;

        self.instructions.borrow_mut().push(RectangleInstruction::Solid {
            screen_position,
            screen_size,
            color,
        });
    }

    fn render_indexed(
        &self,
        texture: Arc<Texture>,
        screen_position: ScreenPosition,
        screen_size: ScreenSize,
        color: Color,
        column_count: usize,
        cell_index: usize,
        smooth: bool,
    ) {
        let screen_position = ScreenPosition {
            left: screen_position.left / self.window_size.width,
            top: screen_position.top / self.window_size.height,
        };

        let screen_size = ScreenSize {
            width: screen_size.width / self.window_size.width,
            height: screen_size.height / self.window_size.height,
        };

        let unit = 1.0 / column_count as f32;
        let offset_x = unit * (cell_index % column_count) as f32;
        let offset_y = unit * (cell_index / column_count) as f32;
        let texture_position = Vector2::new(offset_x, offset_y);
        let texture_size = Vector2::new(unit, unit);

        self.instructions.borrow_mut().push(RectangleInstruction::Sprite {
            screen_position,
            screen_size,
            color,
            texture_position,
            texture_size,
            linear_filtering: smooth,
            texture,
        });
    }
}

#[cfg(feature = "debug")]
impl MarkerRenderer for GameInterfaceRenderer {
    fn render_marker(&mut self, camera: &dyn Camera, marker_identifier: MarkerIdentifier, position: Point3<f32>, hovered: bool) {
        let (top_left_position, bottom_right_position) = camera.billboard_coordinates(position, MarkerIdentifier::SIZE);

        if top_left_position.w >= 0.1 && bottom_right_position.w >= 0.1 {
            let (screen_position, screen_size) = camera.screen_position_size(top_left_position, bottom_right_position);

            let (texture, color) = match marker_identifier {
                MarkerIdentifier::Object(..) if hovered => (&self.object_marker_texture, Color::rgb_u8(235, 180, 52)),
                MarkerIdentifier::Object(..) => (&self.object_marker_texture, Color::rgb_u8(235, 103, 52)),
                MarkerIdentifier::LightSource(..) if hovered => (&self.light_source_marker_texture, Color::rgb_u8(150, 52, 235)),
                MarkerIdentifier::LightSource(..) => (&self.light_source_marker_texture, Color::rgb_u8(52, 235, 217)),
                MarkerIdentifier::SoundSource(..) if hovered => (&self.sound_source_marker_texture, Color::rgb_u8(128, 52, 235)),
                MarkerIdentifier::SoundSource(..) => (&self.sound_source_marker_texture, Color::rgb_u8(235, 52, 140)),
                MarkerIdentifier::EffectSource(..) if hovered => (&self.effect_source_marker_texture, Color::rgb_u8(235, 52, 52)),
                MarkerIdentifier::EffectSource(..) => (&self.effect_source_marker_texture, Color::rgb_u8(52, 235, 156)),
                MarkerIdentifier::Particle(..) if hovered => return,
                MarkerIdentifier::Particle(..) => return,
                MarkerIdentifier::Entity(..) if hovered => (&self.entity_marker_texture, Color::rgb_u8(235, 92, 52)),
                MarkerIdentifier::Entity(..) => (&self.entity_marker_texture, Color::rgb_u8(189, 235, 52)),
                MarkerIdentifier::Shadow(..) if hovered => (&self.shadow_marker_texture, Color::rgb_u8(200, 200, 200)),
                MarkerIdentifier::Shadow(..) => (&self.shadow_marker_texture, Color::rgb_u8(170, 170, 170)),
            };

            self.instructions.borrow_mut().push(RectangleInstruction::Sdf {
                screen_position,
                screen_size,
                color,
                texture_position: Vector2::new(0.0, 0.0),
                texture_size: Vector2::new(1.0, 1.0),
                texture: texture.clone(),
            });
        }
    }
}
