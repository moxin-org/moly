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
}
