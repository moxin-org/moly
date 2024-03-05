use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import makepad_draw::shader::std::*;

    ICON_SEARCH = dep("crate://self/resources/icons/search.svg")

    SearchBar = <View> {
        width: Fill,
        height: 200,

        flow: Down,
        spacing: 30,
        align: {x: 0.5, y: 0.5},

        show_bg: true,

        // TODO Work a bit to have a radial gradient rather than a horizontal one
        draw_bg: {
            instance color2: #AF56DA55,
            instance dither: 1.0
            fn get_color(self) -> vec4 {
                let dither = Math::random_2d(self.pos.xy) * 0.04 * self.dither;
                return mix(self.color, self.color2, self.pos.x + dither)
            }

            fn pixel(self) -> vec4 {
                return Pal::premul(self.get_color())
            }
        }

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 16},
                color: #000
            }
            text: "Discover, download, and run local LLMs"
        }

        <RoundedView> {
            width: Fit,
            height: Fit,

            show_bg: true,
            draw_bg: {
                color: #fff
            }

            padding: {top: 6, bottom: 6, left: 20, right: 20}

            spacing: 4,
            align: {x: 0.0, y: 0.5},

            draw_bg: {
                radius: 10.0,
                border_color: #D0D5DD,
                border_width: 1.0,
            }

            <Icon> {
                draw_icon: {
                    svg_file: (ICON_SEARCH),
                    fn get_color(self) -> vec4 {
                        return #666;
                    }
                }
                icon_walk: {width: 24, height: 24}
            }

            <TextInput> {
                width: 800,
                height: Fit,

                empty_message: "Search Model by Keyword"
                draw_bg: {
                    color: #fff
                }
                draw_text: {
                    text_style:<REGULAR_FONT>{font_size: 14},
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
            }
        }
    }
}