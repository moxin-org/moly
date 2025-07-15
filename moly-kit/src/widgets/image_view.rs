use makepad_widgets::{
    image_cache::{ImageBuffer, ImageError},
    *,
};

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub ImageView = {{ImageView}} {
        align: {x: 0.5, y: 0.5},
        image = <Image> {width: 0, height: 0}
    }
}

/// A wrapped image widget, where it's inner [`Image`] is calculated to an exact size.
///
/// Therefore is affected by certain properties in its wrapper [`View`] such as `align`
/// or `padding` instead of being always `Fill` with changes in the shader.
#[derive(Live, Widget, LiveHook)]
pub struct ImageView {
    #[deref]
    deref: View,

    // TODO: Make an enum with `Contain` and `Cover` variants.
    #[live]
    pub contain: bool,

    #[rust]
    texture: Option<Texture>,
}

impl Widget for ImageView {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Meassure the surroundings.
        let rect = cx.peek_walk_turtle(walk);
        let available_width = rect.size.x;
        let available_height = rect.size.y;

        // Meassure the image size.
        let dpi = cx.current_dpi_factor();
        let (image_width, image_height) = self.image_size(cx);
        let image_width = image_width as f64 * dpi;
        let image_height = image_height as f64 * dpi;

        // Calculate the "stretch" factor.
        let scale_x = available_width / image_width;
        let scale_y = available_height / image_height;

        // Scale the image depending on if should "contain" or "cover".
        let scale = if self.contain {
            // Scale down so the whole image fits inside the available space.
            // Will never scale up.
            scale_x.min(scale_y).clamp(0.0, 1.0)
        } else {
            // Scale up so the whole available space is covered by the image.
            // Will always scale up.
            scale_x.max(scale_y)
        };

        // Calculate the final exact size for the image.
        let scaled_width = image_width * scale;
        let scaled_height = image_height * scale;

        // Apply the new exact size to the image.
        self.image_ref().apply_over(
            cx,
            live! {
                width: (scaled_width),
                height: (scaled_height),
            },
        );

        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope)
    }
}

impl ImageView {
    pub fn load_png(&mut self, cx: &mut Cx, data: &[u8]) -> Result<(), ImageError> {
        self.load_buffer(cx, ImageBuffer::from_png(data)?);
        Ok(())
    }

    pub fn load_jpeg(&mut self, cx: &mut Cx, data: &[u8]) -> Result<(), ImageError> {
        self.load_buffer(cx, ImageBuffer::from_jpg(data)?);
        Ok(())
    }

    pub fn load_with_contet_type(
        &mut self,
        cx: &mut Cx,
        data: &[u8],
        content_type: &str,
    ) -> Result<(), ImageError> {
        // This is esentially double checking in the function and in the match,
        // but this way we can catch inconsistencies between both.
        if can_load(content_type) {
            match content_type {
                "image/png" => self.load_png(cx, data),
                "image/jpeg" => self.load_jpeg(cx, data),
                _ => Err(ImageError::UnsupportedFormat),
            }
        } else {
            Err(ImageError::UnsupportedFormat)
        }
    }

    fn load_buffer(&mut self, cx: &mut Cx, buffer: ImageBuffer) {
        let texture = buffer.into_new_texture(cx);
        self.set_texture(cx, Some(texture));
    }

    pub fn set_texture(&mut self, cx: &mut Cx, texture: Option<Texture>) {
        self.texture = texture;
        self.image_ref().set_texture(cx, self.texture.clone());
    }

    pub fn get_texture(&self) -> Option<Texture> {
        self.texture.clone()
    }

    fn image_ref(&self) -> ImageRef {
        self.image(id!(image))
    }

    fn image_size(&self, cx: &mut Cx) -> (usize, usize) {
        self.texture
            .as_ref()
            .and_then(|t| t.get_format(cx).vec_width_height())
            .unwrap_or((0, 0))
    }
}

/// If this image widget supports the given content type.
pub fn can_load(content_type: &str) -> bool {
    matches!(content_type, "image/png" | "image/jpeg")
}
