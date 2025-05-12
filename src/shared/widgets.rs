use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    
    use crate::shared::styles::*;

    pub VerticalFiller = <View> {
        width: Fill,
        height: 1,
    }

    pub HorizontalFiller = <View> {
        width: 1,
        height: Fill,
    }

    pub Line = <View> {
        width: Fill,
        height: 1,
        show_bg: true,
        draw_bg: {
            color: #D9D9D9
        }
    }

    pub FadeView = <CachedView> {
        draw_bg: {
            instance opacity: 1.0

            fn pixel(self) -> vec4 {
                let color = sample2d_rt(self.image, self.pos * self.scale + self.shift);
                return Pal::premul(vec4(color.xyz, color.w * self.opacity))
            }
        }
    }

    pub AttributeTag = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        spacing: 5,
        draw_bg: {
            instance border_radius: 2.0,
        }

        attr_name = <Label> {
            draw_text: {
                wrap: Word
                text_style: <REGULAR_FONT>{font_size: 8},
                color: #x0
            }
        }
    }

    pub SIDEBAR_FONT_COLOR = #1A2533
    pub SIDEBAR_FONT_COLOR_HOVER = (MAIN_BG_COLOR)
    pub SIDEBAR_FONT_COLOR_SELECTED = (MAIN_BG_COLOR)

    pub SIDEBAR_BG_COLOR_SELECTED = #344054    
    pub SIDEBAR_BG_COLOR_HOVER = #677483
    
    pub SidebarMenuButton = <RadioButton> {
        width: 70,
        height: Fit,
        padding: 8, margin: 0,
        flow: Down, spacing: 8.0, align: {x: 0.5, y: 0.5}

        icon_walk: {margin: 0, width: 25, height: 25}
        label_walk: {margin: 0}

        draw_bg: {
            radio_type: Tab,

            instance border_size: 0.0
            instance border_color_1: #0000
            instance inset: vec4(0.0, 0.0, 0.0, 0.0)
            instance border_radius: 3.5

            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        (MAIN_BG_COLOR_DARK), // TODO(Julian): change to MAIN_BG_COLOR and add shadow
                        (SIDEBAR_BG_COLOR_HOVER),
                        self.hover
                    ),
                    (SIDEBAR_BG_COLOR_SELECTED),
                    self.active
                )
            }

            fn get_border_color(self) -> vec4 {
                return self.border_color_1
            }

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                sdf.box(
                    self.inset.x + self.border_size,
                    self.inset.y + self.border_size,
                    self.rect_size.x - (self.inset.x + self.inset.z + self.border_size * 2.0),
                    self.rect_size.y - (self.inset.y + self.inset.w + self.border_size * 2.0),
                    max(1.0, self.border_radius)
                )
                sdf.fill_keep(self.get_color())
                if self.border_size > 0.0 {
                    sdf.stroke(self.get_border_color(), self.border_size)
                }
                return sdf.result;
            }
        }

        draw_text: {
            color: (SIDEBAR_FONT_COLOR)
            color_hover: (SIDEBAR_FONT_COLOR_HOVER)
            color_active: (SIDEBAR_FONT_COLOR_SELECTED)

            text_style: <BOLD_FONT>{font_size: 9}

            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        self.color,
                        self.color_hover,
                        self.hover
                    ),
                    self.color_active,
                    self.active
                )
            }
        }

        draw_icon: {
            instance color: (SIDEBAR_FONT_COLOR)
            instance color_hover: (SIDEBAR_FONT_COLOR_HOVER)
            instance color_active: (SIDEBAR_FONT_COLOR_SELECTED)
            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        self.color,
                        self.color_hover,
                        self.focus
                    ),
                    self.color_active,
                    self.active
                )
            }
        }
    }

    // Customized button widget, based on the RoundedView shaders with some modifications
    // which is a better fit with our application UI design
    pub MolyButton = <Button> {
        draw_bg: {
            instance color: #0000
            instance color_hover: #fff
            instance border_size: 1.0
            instance border_color_1: #0000
            instance border_color_hover: #fff
            instance border_radius: 2.5

            fn get_color(self) -> vec4 {
                return mix(self.color, mix(self.color, self.color_hover, 0.2), self.hover)
            }

            fn get_border_color(self) -> vec4 {
                return mix(self.border_color_1, mix(self.border_color_1, self.border_color_hover, 0.2), self.hover)
            }

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                sdf.box(
                    self.border_size,
                    self.border_size,
                    self.rect_size.x - (self.border_size * 2.0),
                    self.rect_size.y - (self.border_size * 2.0),
                    max(1.0, self.border_radius)
                )
                sdf.fill_keep(self.get_color())
                if self.border_size > 0.0 {
                    sdf.stroke(self.get_border_color(), self.border_size)
                }
                return sdf.result;
            }
        }

        draw_icon: {
            instance color: #fff
            instance color_hover: #000
            uniform rotation_angle: 0.0,

            fn get_color(self) -> vec4 {
                return mix(self.color, mix(self.color, self.color_hover, 0.2), self.hover)
            }

            // Support rotation of the icon
            fn clip_and_transform_vertex(self, rect_pos: vec2, rect_size: vec2) -> vec4 {
                let clipped: vec2 = clamp(
                    self.geom_pos * rect_size + rect_pos,
                    self.draw_clip.xy,
                    self.draw_clip.zw
                )
                self.pos = (clipped - rect_pos) / rect_size

                // Calculate the texture coordinates based on the rotation angle
                let angle_rad = self.rotation_angle * 3.14159265359 / 180.0;
                let cos_angle = cos(angle_rad);
                let sin_angle = sin(angle_rad);
                let rot_matrix = mat2(
                    cos_angle, -sin_angle,
                    sin_angle, cos_angle
                );
                self.tex_coord1 = mix(
                    self.icon_t1.xy,
                    self.icon_t2.xy,
                    (rot_matrix * (self.pos.xy - vec2(0.5))) + vec2(0.5)
                );

                return self.camera_projection * (self.camera_view * (self.view_transform * vec4(
                    clipped.x,
                    clipped.y,
                    self.draw_depth + self.draw_zbias,
                    1.
                )))
            }
        }
        icon_walk: {width: 14, height: 14}

        draw_text: {
            color: #fff,
            text_style: <REGULAR_FONT>{font_size: 9},
            fn get_color(self) -> vec4 {
                return self.color;
            }
        }

        reset_hover_on_click: true
    }

    pub MolyRadioButtonTab = <RadioButtonTab> {
        padding: 10,

        draw_bg: {
            uniform border_radius: 3.0
            uniform border_size: 0.0
            instance color: (THEME_COLOR_TEXT)
            instance color_hover: (THEME_COLOR_TEXT_HOVER)

            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        self.color,
                        self.color_hover,
                        self.hover
                    ),
                    self.color_active,
                    self.active
                )
            }

            fn get_border_color(self) -> vec4 {
                return self.border_color_1;
            }

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                match self.radio_type {
                    RadioType::Tab => {
                        sdf.box(
                            self.border_size,
                            self.border_size,
                            self.rect_size.x - (self.border_size * 2.0),
                            self.rect_size.y - (self.border_size * 2.0),
                            max(1.0, self.border_radius)
                        )
                        sdf.fill_keep(self.get_color())
                        if self.border_size > 0.0 {
                            sdf.stroke(self.get_border_color(), self.border_size)
                        }
                    }
                }
                return sdf.result
            }
        }

        draw_text: {
            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        self.color,
                        self.color_hover,
                        self.hover
                    ),
                    self.color_active,
                    self.active
                )
            }
        }
    }

    // Customized text input
    // Removes shadows, focus highlight and the dark theme colors
    pub MolyTextInput = <TextInput> {
        draw_text: {
            text_style:<REGULAR_FONT>{font_size: 12},
            fn get_color(self) -> vec4 {
                return #555
            }
        }

        // TODO find a way to override colors
        draw_cursor: {
            instance focus: 0.0
            uniform border_radius: 0.5
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(
                    0.,
                    0.,
                    self.rect_size.x,
                    self.rect_size.y,
                    self.border_radius
                )
                sdf.fill(mix(#fff, #bbb, self.focus));
                return sdf.result
            }
        }

        // TODO find a way to override colors
        draw_selection: {
            instance hover: 0.0
            instance focus: 0.0
            uniform border_radius: 2.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(
                    0.,
                    0.,
                    self.rect_size.x,
                    self.rect_size.y,
                    self.border_radius
                )
                sdf.fill(mix(#eee, #ddd, self.focus)); // Pad color
                return sdf.result
            }
        }

        draw_bg: {
            color: #fff
            instance border_radius: 2.0
            instance border_size: 0.0
            instance border_color_1: #3
            instance inset: vec4(0.0, 0.0, 0.0, 0.0)

            fn get_color(self) -> vec4 {
                return self.color
            }

            fn get_border_color(self) -> vec4 {
                return self.border_color_1
            }

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                sdf.box(
                    self.inset.x + self.border_size,
                    self.inset.y + self.border_size,
                    self.rect_size.x - (self.inset.x + self.inset.z + self.border_size * 2.0),
                    self.rect_size.y - (self.inset.y + self.inset.w + self.border_size * 2.0),
                    max(1.0, self.border_radius)
                )
                sdf.fill_keep(self.get_color())
                if self.border_size > 0.0 {
                    sdf.stroke(self.get_border_color(), self.border_size)
                }
                return sdf.result;
            }
        }
    }

    pub MolySlider =  <Slider> {
        height: 40
        width: Fill
        draw_text: {
            // TODO: The text weight should be 500 (semi bold, not fully bold).
            text_style: <BOLD_FONT>{font_size: 10},
            color: #000
        }
        text_input: {
            draw_text: {
                text_style: <BOLD_FONT>{font_size: 11},
                fn get_color(self) -> vec4 {
                    return #000;
                }
            }
        }
        draw_slider: {
            instance bipolar: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)

                let ball_radius = 10.0;
                let ball_border = 2.0;
                let padding_top = 29.0;
                let padding_x = 5.0;
                let rail_height = 4.0;

                let rail_width = self.rect_size.x;
                let rail_padding_x = padding_x + ball_radius / 2;
                let ball_rel_x = self.slide_pos;
                let ball_abs_x = ball_rel_x * (rail_width - 2.0 * rail_padding_x) + rail_padding_x;

                // The rail
                sdf.move_to(0 + padding_x, padding_top);
                sdf.line_to(self.rect_size.x - padding_x, padding_top);
                sdf.stroke(#D9D9D9, rail_height);

                // The filler
                sdf.move_to(0 + padding_x, padding_top);
                sdf.line_to(ball_abs_x, padding_top);
                sdf.stroke(#15859A, rail_height);

                // The moving ball
                sdf.circle(ball_abs_x, padding_top, ball_radius);
                sdf.fill(#15859A);
                sdf.circle(ball_abs_x, padding_top, ball_radius - ball_border);
                sdf.fill(#fff);


                return sdf.result;
            }
        }
    }

    pub MolySwitch = <Toggle> {
        // U+200e as text.
        // Nasty trick cause not setting `text` nor using a simple space works to
        // render the widget without label.
        text:"‎"
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                let pill_padding = 2.0;
                let pill_color_off = #D9D9D9;
                let pill_color_on = #429E92

                let pill_radius = self.rect_size.y * 0.5;
                let ball_radius = pill_radius - pill_padding;

                // Left side of the pill
                sdf.circle(pill_radius, pill_radius, pill_radius);
                sdf.fill(mix(pill_color_off, pill_color_on, self.active));

                // Right side of the pill
                sdf.circle(self.rect_size.x - pill_radius, pill_radius, pill_radius);
                sdf.fill(mix(pill_color_off, pill_color_on, self.active));

                // The union/middle of the pill
                sdf.rect(pill_radius, 0.0, self.rect_size.x - 2.0 * pill_radius, self.rect_size.y);
                sdf.fill(mix(pill_color_off, pill_color_on, self.active));

                // The moving ball
                sdf.circle(pill_padding + ball_radius + self.active * (self.rect_size.x - 2.0 * ball_radius - 2.0 * pill_padding), pill_radius, ball_radius);
                sdf.fill(#fff);

                return sdf.result;
            }
        }
    }

    pub TogglePanelButton = <MolyButton> {
        width: Fit,
        height: Fit,
        icon_walk: {width: 20, height: 20},
        draw_icon: {
            fn get_color(self) -> vec4 {
                return #475467;
            }
        }
    }

    pub MolyTogglePanel = <TogglePanel> {
        persistent_content = {
            default = {
                open = <TogglePanelButton> {
                    visible: false,
                    draw_icon: {
                        svg_file: (TOGGLE_PANEL_OPEN_ICON)
                    }
                }
                close = <TogglePanelButton> {
                    draw_icon: {
                        svg_file: (TOGGLE_PANEL_CLOSE_ICON)
                    }
                }
            }
        }
    }
}
