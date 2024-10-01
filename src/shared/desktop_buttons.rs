use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    // Overrides the dark theme colors of the desktop buttons.
    // This is a temporaty fix until theme system allows for more flexible color overrides.
    MolyDesktopButton = <DesktopButton> {
        draw_bg: {
            instance stroke_color: #5
            instance button_color: #f
            instance hover_color: #d5
            instance pressed_color: #c5
            instance close_hover_color: #e00
            instance close_hover_stroke_color: #f
            instance close_pressed_color: #c00

            fn get_bg_color(self, base_color: vec4, hover_color: vec4, pressed_color: vec4) -> vec4 {
                return mix(base_color, mix(hover_color, pressed_color, self.pressed), self.hover);
            }
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.aa *= 3.0;
                let sz = 4.5;
                let c = self.rect_size * vec2(0.5, 0.5);

                match self.button_type {
                    DesktopButtonType::WindowsMin => {
                        sdf.clear(self.get_bg_color(self.button_color, self.hover_color, self.pressed_color));
                        sdf.move_to(c.x - sz, c.y);
                        sdf.line_to(c.x + sz, c.y);
                        sdf.stroke(self.stroke_color, 0.5 + 0.5 * self.dpi_dilate);
                        return sdf.result;
                    }
                    DesktopButtonType::WindowsMax => {
                        sdf.clear(self.get_bg_color(self.button_color, self.hover_color, self.pressed_color));
                        sdf.rect(c.x - sz, c.y - sz, 2. * sz, 2. * sz);
                        sdf.stroke(self.stroke_color, 0.3 + 0.5 * self.dpi_dilate);
                        return sdf.result;
                    }
                    DesktopButtonType::WindowsMaxToggled => {
                        let clear = self.get_bg_color(self.button_color, self.hover_color, self.pressed_color);
                        sdf.clear(clear);
                        let sz = 3.5;
                        sdf.rect(c.x - sz + 1., c.y - sz - 1., 2. * sz, 2. * sz);
                        sdf.stroke(self.stroke_color, 0.3 + 0.5 * self.dpi_dilate);
                        sdf.rect(c.x - sz - 1., c.y - sz + 1., 2. * sz, 2. * sz);
                        sdf.fill_keep(clear);
                        sdf.stroke(self.stroke_color, 0.3 + 0.5 * self.dpi_dilate);
                        return sdf.result;
                    }
                    DesktopButtonType::WindowsClose => {
                        sdf.clear(self.get_bg_color(self.button_color, self.close_hover_color, self.close_pressed_color));
                        sdf.move_to(c.x - sz, c.y - sz);
                        sdf.line_to(c.x + sz, c.y + sz);
                        sdf.move_to(c.x - sz, c.y + sz);
                        sdf.line_to(c.x + sz, c.y - sz);
                        sdf.stroke(mix(self.stroke_color, self.close_hover_stroke_color, self.hover), 0.5 + 0.5 * self.dpi_dilate);
                        return sdf.result;
                    }
                    DesktopButtonType::XRMode => {
                        sdf.clear(mix(THEME_COLOR_APP_CAPTION_BAR, mix(#0aa, #077, self.pressed), self.hover));
                        let w = 12.;
                        let h = 8.;
                        sdf.box(c.x - w, c.y - h, 2. * w, 2. * h, 2.);
                        // subtract 2 eyes
                        sdf.circle(c.x - 5.5, c.y, 3.5);
                        sdf.subtract();
                        sdf.circle(c.x + 5.5, c.y, 3.5);
                        sdf.subtract();
                        sdf.circle(c.x, c.y + h - 0.75, 2.5);
                        sdf.subtract();
                        sdf.fill(#8);

                        return sdf.result;
                    }
                    DesktopButtonType::Fullscreen => {
                        sz = 8.;
                        sdf.clear(mix(#3, mix(#6, #9, self.pressed), self.hover));
                        sdf.rect(c.x - sz, c.y - sz, 2. * sz, 2. * sz);
                        sdf.rect(c.x - sz + 1.5, c.y - sz + 1.5, 2. * (sz - 1.5), 2. * (sz - 1.5));
                        sdf.subtract();
                        sdf.rect(c.x - sz + 4., c.y - sz - 2., 2. * (sz - 4.), 2. * (sz + 2.));
                        sdf.subtract();
                        sdf.rect(c.x - sz - 2., c.y - sz + 4., 2. * (sz + 2.), 2. * (sz - 4.));
                        sdf.subtract();
                        sdf.fill(self.stroke_color); //, 0.5 + 0.5 * dpi_dilate);

                        return sdf.result;
                    }
                }
                return #f00;
            }
        }
    }
}
