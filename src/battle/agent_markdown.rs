use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;

    import crate::chat::chat_line::MessageText;

    BaseMarkdown = <MessageText> {
        padding: 0,
        margin: 0,
        // Workaround: This property causes an unintended initial space so let's disable it.
        paragraph_spacing: 0,
    }

    AgentMarkdown = <BaseMarkdown> {
        draw_normal: {
            color: (#000),
        }
        draw_italic: {
            color: (#000),
        }
        draw_bold: {
            color: (#000),
        }
        draw_bold_italic: {
            color: (#000),
        }
        draw_fixed: {
            color: (#000),
        }
        draw_block: {
            line_color: (#000)
            sep_color: (#EDEDED)
            quote_bg_color: (#EDEDED)
            quote_fg_color: (#969696)
            code_color: (#EDEDED)
        }
    }
}
