use makepad_widgets::*;
use pulldown_cmark::{Options, Parser};

live_design! {
    link widgets;
    use link::widgets::*;
    use link::theme::*;

    pub CustomMarkdownBase = {{CustomMarkdown}} <Html> {}

    pub CustomMarkdown = <CustomMarkdownBase> {}
}

#[derive(Live, LiveHook, Widget)]
pub struct CustomMarkdown {
    #[deref]
    inner_html: Html,
    #[rust]
    md_text: String,
}

impl Widget for CustomMarkdown {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.inner_html.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.inner_html.draw_walk(cx, scope, walk)
    }

    fn text(&self) -> String {
        self.md_text.clone()
    }

    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        self.md_text = v.to_string();

        let html_text = self.parse_md_to_html(&self.md_text);
        self.inner_html.set_text(cx, &html_text);

        self.redraw(cx);
    }
}

impl CustomMarkdown {
    /// Converts markdown text to HTML with special handling for code blocks.
    /// 
    /// This function addresses several issues with pulldown-cmark HTML output:
    /// 1. Removes unnecessary newlines from regular HTML elements (like lists, paragraphs)
    /// 2. Preserves newlines within code blocks for proper code formatting
    /// 3. Removes empty spaces that appear after code blocks
    /// 
    /// The approach splits the HTML at code block boundaries, processes each section
    /// appropriately, and then rejoins them.
    fn parse_md_to_html(&self, md_text: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&md_text, options);

        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);
        
        // Split HTML by code blocks to preserve newlines inside them
        let parts: Vec<&str> = html_output.split("<pre><code").collect();
        let mut result = String::new();
        
        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                // First part (before any code block) - remove all newlines
                result.push_str(&part.replace("\n", ""));
            } else {
                // This part starts with the inside of a code block
                // Find where the code block ends
                if let Some(end_idx) = part.find("</code></pre>") {
                    let (code_block, rest) = part.split_at(end_idx + 13); // 13 is the length of "</code></pre>"
                    
                    // Add back the opening tag we removed during split
                    result.push_str("<pre><code");
                    // Add the code block with newlines preserved
                    result.push_str(code_block);
                    
                    // Remove newlines and any extra whitespace that follows code blocks
                    // First normalize all newlines and spaces after the code block
                    let clean_rest = rest.replace("\n", " ")
                        .replace("  ", " "); // Collapse multiple spaces
                    
                    // Then trim any leading space that might occur right after the code block
                    let clean_rest = if clean_rest.starts_with(' ') {
                        clean_rest[1..].to_string()
                    } else {
                        clean_rest
                    };
                    
                    result.push_str(&clean_rest);
                } else {
                    // Something went wrong, just add back with the tag
                    result.push_str("<pre><code");
                    result.push_str(part);
                }
            }
        }
        
        // println!("Final html output: {:#?}", result);
        result
    }
}
