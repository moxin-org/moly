use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::shared::external_link::*;

    pub ModelLink = <View> {
        width: Fit,
        height: Fit,
        flow: Down,
        link = <LinkLabel> {
            width: Fit,
            margin: 2,
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9},
                fn get_color(self) -> vec4 {
                    return mix(
                        mix(
                            MODEL_LINK_FONT_COLOR,
                            MODEL_LINK_FONT_COLOR,
                            self.hover
                        ),
                        MODEL_LINK_FONT_COLOR,
                        self.down
                    )
                }
            }
        }
        underline = <Line> {
            width: Fill,
            height: 1,
            show_bg: true,
            draw_bg: {
                color: (MODEL_LINK_FONT_COLOR)
            }
        }
    }

    pub ModelAttributeTag = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        spacing: 5,
        draw_bg: {
            instance border_radius: 3.0,
        }

        attr_name = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #x0
            }
        }

        attr_value = <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 9},
                color: #x0
            }
        }
    }

    pub ModelAttributes = <View> {
        width: Fit,
        height: Fit,
        spacing: 8,

        model_size_tag = <ModelAttributeTag> {
            draw_bg: { color: #D4E6F7 },
            attr_name = { text: "Model Size" }
        }

        model_requires_tag = <ModelAttributeTag> {
            draw_bg: { color: #D6F5EB },
            attr_name = { text: "Requires" }
        }

        model_architecture_tag = <ModelAttributeTag> {
            draw_bg: { color: #F0D6F5 },
            attr_name = { text: "Architecture" }
        }
    }
}
