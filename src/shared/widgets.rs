use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;
    import crate::shared::styles::*;

    VerticalFiller = <View> {
        width: Fill,
        height: 1,
    }

    HorizontalFiller = <View> {
        width: 1,
        height: Fill,
    }

    Line = <View> {
        width: Fill,
        height: 1,
        show_bg: true,
        draw_bg: {
            color: #D9D9D9
        }
    }

    FadeView = <CachedView> {
        draw_bg: {
            instance opacity: 1.0

            fn pixel(self) -> vec4 {
                let color = sample2d_rt(self.image, self.pos * self.scale + self.shift) + vec4(self.marked, 0.0, 0.0, 0.0);
                return Pal::premul(vec4(color.xyz, color.w * self.opacity))
            }
        }
    }

    AttributeTag = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        spacing: 5,
        draw_bg: {
            instance radius: 2.0,
        }

        attr_name = <Label> {
            draw_text: {
                wrap: Word
                text_style: <REGULAR_FONT>{font_size: 8},
                color: #x0
            }
        }
    }

    SIDEBAR_FONT_COLOR = #344054
    SIDEBAR_FONT_COLOR_HOVER = #344054
    SIDEBAR_FONT_COLOR_SELECTED = #127487

    SIDEBAR_BG_COLOR_HOVER = #E2F1F199
    SIDEBAR_BG_COLOR_SELECTED = #E2F1F199

    SidebarMenuButton = <RadioButton> {
        width: 80,
        height: 70,
        padding: 0, margin: 0,
        flow: Down, spacing: 8.0, align: {x: 0.5, y: 0.5}

        icon_walk: {margin: 0, width: 30, height: 30}
        label_walk: {margin: 0}

        draw_radio: {
            radio_type: Tab,

            instance border_width: 0.0
            instance border_color: #0000
            instance inset: vec4(0.0, 0.0, 0.0, 0.0)
            instance radius: 2.5

            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        (SIDEBAR_BG_COLOR),
                        (SIDEBAR_BG_COLOR_HOVER),
                        self.hover
                    ),
                    (SIDEBAR_BG_COLOR_SELECTED),
                    self.selected
                )
            }

            fn get_border_color(self) -> vec4 {
                return self.border_color
            }

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                sdf.box(
                    self.inset.x + self.border_width,
                    self.inset.y + self.border_width,
                    self.rect_size.x - (self.inset.x + self.inset.z + self.border_width * 2.0),
                    self.rect_size.y - (self.inset.y + self.inset.w + self.border_width * 2.0),
                    max(1.0, self.radius)
                )
                sdf.fill_keep(self.get_color())
                if self.border_width > 0.0 {
                    sdf.stroke(self.get_border_color(), self.border_width)
                }
                return sdf.result;
            }
        }

        draw_text: {
            color_unselected: (SIDEBAR_FONT_COLOR)
            color_unselected_hover: (SIDEBAR_FONT_COLOR_HOVER)
            color_selected: (SIDEBAR_FONT_COLOR_SELECTED)

            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        self.color_unselected,
                        self.color_unselected_hover,
                        self.hover
                    ),
                    self.color_selected,
                    self.selected
                )
            }
        }

        draw_icon: {
            instance color_unselected: (SIDEBAR_FONT_COLOR)
            instance color_unselected_hover: (SIDEBAR_FONT_COLOR_HOVER)
            instance color_selected: (SIDEBAR_FONT_COLOR_SELECTED)
            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        self.color_unselected,
                        self.color_unselected_hover,
                        self.hover
                    ),
                    self.color_selected,
                    self.selected
                )
            }
        }
    }

    // Customized button widget, based on the RoundedView shaders with some modifications
    // which is a better fit with our application UI design
    MolyButton = <Button> {
        draw_bg: {
            instance color: #0000
            instance color_hover: #fff
            instance border_width: 1.0
            instance border_color: #0000
            instance border_color_hover: #fff
            instance radius: 2.5

            fn get_color(self) -> vec4 {
                return mix(self.color, mix(self.color, self.color_hover, 0.2), self.hover)
            }

            fn get_border_color(self) -> vec4 {
                return mix(self.border_color, mix(self.border_color, self.border_color_hover, 0.2), self.hover)
            }

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - (self.border_width * 2.0),
                    self.rect_size.y - (self.border_width * 2.0),
                    max(1.0, self.radius)
                )
                sdf.fill_keep(self.get_color())
                if self.border_width > 0.0 {
                    sdf.stroke(self.get_border_color(), self.border_width)
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
            text_style: <REGULAR_FONT>{font_size: 9},
            fn get_color(self) -> vec4 {
                return self.color;
            }
        }

        reset_hover_on_click: true
    }

    MolyRadioButtonTab = <RadioButtonTab> {
        padding: 10,

        draw_radio: {
            uniform radius: 3.0
            uniform border_width: 0.0
            instance color_unselected: (THEME_COLOR_TEXT_DEFAULT)
            instance color_unselected_hover: (THEME_COLOR_TEXT_HOVER)
            instance color_selected: (THEME_COLOR_TEXT_SELECTED)
            instance border_color: (THEME_COLOR_TEXT_SELECTED)

            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        self.color_unselected,
                        self.color_unselected_hover,
                        self.hover
                    ),
                    self.color_selected,
                    self.selected
                )
            }

            fn get_border_color(self) -> vec4 {
                return self.border_color;
            }

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                match self.radio_type {
                    RadioType::Tab => {
                        sdf.box(
                            self.border_width,
                            self.border_width,
                            self.rect_size.x - (self.border_width * 2.0),
                            self.rect_size.y - (self.border_width * 2.0),
                            max(1.0, self.radius)
                        )
                        sdf.fill_keep(self.get_color())
                        if self.border_width > 0.0 {
                            sdf.stroke(self.get_border_color(), self.border_width)
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
                        self.color_unselected,
                        self.color_unselected_hover,
                        self.hover
                    ),
                    self.color_selected,
                    self.selected
                )
            }
        }
    }

    // Customized text input
    // Removes shadows, focus highlight and the dark theme colors
    MolyTextInput = <TextInput> {
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
            instance radius: 2.0
            instance border_width: 0.0
            instance border_color: #3
            instance inset: vec4(0.0, 0.0, 0.0, 0.0)

            fn get_color(self) -> vec4 {
                return self.color
            }

            fn get_border_color(self) -> vec4 {
                return self.border_color
            }

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                sdf.box(
                    self.inset.x + self.border_width,
                    self.inset.y + self.border_width,
                    self.rect_size.x - (self.inset.x + self.inset.z + self.border_width * 2.0),
                    self.rect_size.y - (self.inset.y + self.inset.w + self.border_width * 2.0),
                    max(1.0, self.radius)
                )
                sdf.fill_keep(self.get_color())
                if self.border_width > 0.0 {
                    sdf.stroke(self.get_border_color(), self.border_width)
                }
                return sdf.result;
            }
        }
    }

    MolySlider =  <Slider> {
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

    MolySwitch = <CheckBoxToggle> {
        // U+200e as text.
        // Nasty trick cause not setting `text` nor using a simple space works to
        // render the widget without label.
        text:"‎"
        draw_check: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                let pill_padding = 2.0;
                let pill_color_off = #D9D9D9;
                let pill_color_on = #15859A;

                let pill_radius = self.rect_size.y * 0.5;
                let ball_radius = pill_radius - pill_padding;

                // Left side of the pill
                sdf.circle(pill_radius, pill_radius, pill_radius);
                sdf.fill(mix(pill_color_off, pill_color_on, self.selected));

                // Right side of the pill
                sdf.circle(self.rect_size.x - pill_radius, pill_radius, pill_radius);
                sdf.fill(mix(pill_color_off, pill_color_on, self.selected));

                // The union/middle of the pill
                sdf.rect(pill_radius, 0.0, self.rect_size.x - 2.0 * pill_radius, self.rect_size.y);
                sdf.fill(mix(pill_color_off, pill_color_on, self.selected));

                // The moving ball
                sdf.circle(pill_padding + ball_radius + self.selected * (self.rect_size.x - 2.0 * ball_radius - 2.0 * pill_padding), pill_radius, ball_radius);
                sdf.fill(#fff);

                return sdf.result;
            }
        }
    }

    TogglePanelButton = <MolyButton> {
        width: Fit,
        height: Fit,
        icon_walk: {width: 20, height: 20},
        draw_icon: {
            fn get_color(self) -> vec4 {
                return #475467;
            }
        }
    }

    MolyTogglePanel = <TogglePanel> {
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
