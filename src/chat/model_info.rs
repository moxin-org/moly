use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    
    use crate::shared::styles::*;
    use crate::chat::shared::ChatAgentAvatar;

    ICON_TICK = dep("crate://self/resources/images/tick.png")

    ModelAttributeTag = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        spacing: 5,
        draw_bg: {
            border_radius: 2.0,
        }

        caption = <Label> {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #1D2939
            }
        }
    }

    pub ModelInfo = <View> {
        width: Fill, height: Fit
        padding: 16,
        spacing: 10,
        align: {x: 0.0, y: 0.5},

        cursor: Hand,

        label = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #000
            }
        }

        architecture_tag = <ModelAttributeTag> {
            draw_bg: {
                color: #DDD7FF,
            }
        }

        params_size_tag = <ModelAttributeTag> {
            draw_bg: {
                color: #D1F4FC,
            }
        }

        file_size_tag = <ModelAttributeTag> {
            caption = {
                draw_text:{
                    color: #000
                }
            }
            draw_bg: {
                color: #fff,
                border_color: #B4B4B4,
                border_size: 1.0,
            }
        }

        icon_tick_tag = <RoundedView> {
            align: {x: 1.0, y: 0.5}, 
            visible: false,
            icon_tick = <Image> {
                width: 14,
                height: 14,
                source: (ICON_TICK),
            }
        }
    }

    pub AgentInfo = <View> {
        width: Fill,
        height: Fit,
        padding: 16,

        align: {x: 0.0, y: 0.5},
        spacing: 10,

        cursor: Hand,

        avatar = <ChatAgentAvatar> {}

        label = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #000
            }
        }

        icon_tick_tag = <RoundedView> {
            align: {x: 1.0, y: 0.5}, 
            visible: false,
            icon_tick = <Image> {
                width: 14,
                height: 14,
                source: (ICON_TICK),
            }
        }
    }
}

