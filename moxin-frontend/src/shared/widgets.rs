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

    AttributeTag = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        spacing: 5,
        draw_bg: {
            instance radius: 2.0,
        }

        attr_name = <Label> {
            draw_text:{
                wrap: Word
                text_style: <REGULAR_FONT>{font_size: 8},
                color: #x0
            }
        }
    }

    SIDEBAR_COLOR = #344054
    SIDEBAR_COLOR_HOVER = #636e82
    SIDEBAR_COLOR_SELECTED = #B258DD

    SidebarMenuButton = <RadioButton> {
        width: 96,
        height: 60,
        padding: 0, margin: 0,
        flow: Down, spacing: 10.0, align: {x: 0.5, y: 0.5}

        icon_walk: {margin: 0, width: 32, height: 32}
        label_walk: {margin: 0}

        draw_radio: {
            radio_type: Tab,

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(
                    self.rect_size.x-2,
                    0.0,
                    self.rect_size.x,
                    self.rect_size.y,
                    0.5
                );
                sdf.fill(
                    mix(
                        mix(
                            #0000,
                            (SIDEBAR_COLOR_HOVER),
                            self.hover
                        ),
                        (SIDEBAR_COLOR_SELECTED),
                        self.selected
                    )
                );
                return sdf.result;
            }
        }

        draw_text: {
            color_unselected: (SIDEBAR_COLOR)
            color_unselected_hover: (SIDEBAR_COLOR_HOVER)
            color_selected: (SIDEBAR_COLOR_SELECTED)

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
            instance color_unselected: (SIDEBAR_COLOR)
            instance color_unselected_hover: (SIDEBAR_COLOR_HOVER)
            instance color_selected: (SIDEBAR_COLOR_SELECTED)
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
    MoxinTextInput = <TextInput> {
        draw_bg: {
            color: #fff
        }
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
        draw_select: {
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
}
