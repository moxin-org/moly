use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    // import crate::shared::styles::*;

    use makepad_code_editor::code_view::CodeView;

    // copied as it is from moly
    MessageText = <Markdown> {
        padding: 0,
        paragraph_spacing: 20.0,
        font_color: #000,
        width: Fill, height: Fit,
        font_size: 10.0,
        code_block = <View> {
            width:Fill,
            height:Fit,
            code_view = <CodeView>{
                editor: {
                    pad_left_top: vec2(10.0,10.0)
                    width: Fill,
                    height: Fit,
                    draw_bg: { color: #3c3c3c },
                }
            }
        }
        use_code_block_widget: true,
        list_item_layout: { padding: {left: 10.0, right:10, top: 6.0, bottom: 0}, }
        list_item_walk:{margin:0, height:Fit, width:Fill}
        code_layout: { padding: {top: 10.0, bottom: 10.0}}
        quote_layout: { padding: {top: 10.0, bottom: 10.0}}

        link = {
            padding: { top: 1, bottom: 0 },
            draw_text: {
                color: #00f,
                color_pressed: #f00,
                color_hover: #0f0,
            }
        }
    }

    BaseMarkdown = <MessageText> {
        padding: 0,
        margin: 0,
        // Workaround: This property causes an unintended initial space so let's disable it.
        paragraph_spacing: 0,
    }

    pub MessageMarkdown = <BaseMarkdown> {
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
