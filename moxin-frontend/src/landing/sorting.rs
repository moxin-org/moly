use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;

    ModelsDropDown = <DropDown> {
        width: Fit
        height: Fit
        padding: {top: 10.0, right: 20.0, bottom: 10.0, left: 10.0}

        popup_menu_position: BelowInput

        draw_text: {
            text_style: <BOLD_FONT> { font_size: 10 },
            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        #000,
                        #000,
                        self.focus
                    ),
                    #000,
                    self.pressed
                )
            }
        }

        popup_menu: {
            width: 220,

            draw_bg: {
                color: #fff,
                border_width: 1.5,
                border_color: #EAECF0,
                radius: 4.0
                blur: 0.0
            }

            menu_item: {
                width: Fill,
                height: Fit

                padding: {left: 20, top: 15, bottom: 15, right: 20}

                draw_bg: {
                    color: #fff,
                    color_selected: #eee9,

                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);

                        sdf.clear(mix(
                            self.color,
                            self.color_selected,
                            self.hover
                        ))

                        let sz = 4.;
                        let dx = 2.0;
                        let c = vec2(0.9 * self.rect_size.x, 0.5 * self.rect_size.y);
                        sdf.move_to(c.x - sz + dx * 0.5, c.y - sz + dx);
                        sdf.line_to(c.x, c.y + sz);
                        sdf.line_to(c.x + sz * 2.0, c.y - sz * 2.0);
                        sdf.stroke(mix(#0000, #0, self.selected), 1.5);

                        return sdf.result;
                    }
                }

                draw_name: {
                    text_style: <BOLD_FONT> { font_size: 10 }
                    instance selected: 0.0
                    instance hover: 0.0
                    fn get_color(self) -> vec4 {
                        return #000;
                    }
                }
            }
        }

        draw_bg: {
            fn get_bg(self, inout sdf: Sdf2d) {
                sdf.box(
                    2,
                    2,
                    self.rect_size.x - 4,
                    self.rect_size.y - 4,
                    4.0
                )
                sdf.stroke_keep(#EAECF0, 2.);
                sdf.fill(#fff);
            }
        }
    }

    Sorting = <View> {
        width: Fit,
        height: Fit,
        align: {x: 0.5, y: 0.5},

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 12},
                color: #667085
            }
            text: "SORT BY"
        }
        
        <ModelsDropDown> {
            width: 220,
            height: Fit,

            margin: { left: 20, right: 40 }

            labels: ["Most Downloads", "Least Downloads", "Most Likes", "Least Likes"]
            values: [MostDownloads, LeastDownloads, MostLikes, LeastLikes]
        }
    }
}