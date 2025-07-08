use makepad_widgets::{
    image_cache::{ImageBuffer, ImageError},
    *,
};

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub ImageContain = {{ImageContain}} {
        align: {x: 0.5, y: 0.5},
        image = <Image> {}
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct ImageContain {
    #[deref]
    deref: View,

    #[rust]
    image_width: usize,

    #[rust]
    image_height: usize,
}

impl Widget for ImageContain {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.peek_walk_turtle(walk);
        let available_width = rect.size.x;
        let available_height = rect.size.y;

        let dpi = cx.current_dpi_factor();
        let image_width = self.image_width as f64 * dpi;
        let image_height = self.image_height as f64 * dpi;

        let scale = if image_width > available_width || image_height > available_height {
            let scale_x = available_width / image_width;
            let scale_y = available_height / image_height;
            scale_x.min(scale_y)
        } else {
            1.0
        };

        let scaled_width = image_width * scale;
        let scaled_height = image_height * scale;

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

impl ImageContain {
    pub fn load_png(&mut self, cx: &mut Cx, data: &[u8]) -> Result<(), ImageError> {
        let buffer = ImageBuffer::from_png(data)?;
        self.image_width = buffer.width;
        self.image_height = buffer.height;
        let texture = buffer.into_new_texture(cx);
        self.image_ref().set_texture(cx, Some(texture));
        Ok(())
    }

    fn image_ref(&self) -> ImageRef {
        self.image(id!(image))
    }
}

impl ImageContainRef {
    /// Immutable access to the underlying [`ImageContain`].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> std::cell::Ref<ImageContain> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [`ImageContain`].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> std::cell::RefMut<ImageContain> {
        self.borrow_mut().unwrap()
    }
}
