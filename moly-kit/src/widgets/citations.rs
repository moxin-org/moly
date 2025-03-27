use std::collections::{HashMap, HashSet};

use crate::utils::{asynchronous::spawn, events::EventExt};
use makepad_widgets::*;
use reqwest::header::{HeaderValue, USER_AGENT};

live_design! {
    use link::theme::*;
    use link::widgets::*;

    BOLD_FONT = {
        font: {path: dep("crate://makepad-widgets/resources/IBMPlexSans-SemiBold.ttf")}
    }

    LINK_ICON = dep("crate://self/resources/link.png")

    LinkPreviewUI = {{LinkPreviewUI}} <RoundedView> {
        cursor: Hand,
        height: 75, width: 180
        flow: Right, spacing: 5
        show_bg: true,
        draw_bg: {
            color: #f2f2f2
            radius: 0
        }
        align: {y: 0.5}
        image_wrapper = <RoundedView> {
            draw_bg: {
                radius: 0
            },
            align: {y: 0.5, x: 0.5},
            width: 75, height: Fill,
            visible: true,
            image = <Image> {
                width: 30, height: 55,
                fit: Vertical,
                source: (LINK_ICON)
            }
        }
        flow_down_wrapper = <View> {
            flow: Down, spacing: 5
            align: {y: 0.5, x: 0.0}
            title = <Label> {
                text: "Loading..."
                draw_text: {
                    text_style: <BOLD_FONT>{},
                    color: #000
                }
            }
            domain = <Label> {
                text: "Loading..."
                draw_text: {
                    color: #000
                }
            }
        }
    }

    pub Citations = {{Citations}} {
        margin: {top: 15}
        height: Fit, width: Fill,
        flow: RightWrap, spacing: 10
        // TODO: We want make these rounded but don't have a straight forward way to have the inner image match the same radius.
        citation_template: <LinkPreviewUI> {}
    }


}

#[derive(Live, LiveHook, Widget)]
pub struct Citations {
    #[deref]
    view: View,

    /// The template for the citation views.
    #[live]
    citation_template: Option<LivePtr>,

    /// The views that represent the citations.
    #[rust]
    link_preview_children: ComponentMap<usize, LinkPreviewUI>,

    /// The citations (URLs) that are currently being rendered.
    #[rust]
    citations: Vec<String>,

    /// Maps the index of the citation to the image blob.
    #[rust]
    image_blobs: HashMap<usize, Vec<u8>>,

    /// Track which images have already been loaded
    #[rust]
    loaded_image_indices: HashSet<usize>,
}

impl Widget for Citations {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for (_citation_id, citation) in self.link_preview_children.iter_mut() {
            citation.handle_event(cx, event, scope);
        }
        self.view.handle_event(cx, event, scope);
        self.ui_runner().handle(cx, event, scope, self);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Without this, margins and paddings would be considered even if not visible
        // because of the turtle.
        if !self.visible {
            return DrawStep::done();
        }

        // TODO: Fix this, currently redrawing on every event
        // And at the same time, the citations are not being redrawn unless there's a user-triggered event like mouse move or window resize.
        cx.begin_turtle(walk, self.layout);
        for (citation_id, citation) in self.link_preview_children.iter_mut() {
            let citation_content = &self.citations[*citation_id];
            let domain = url::Url::parse(citation_content)
                .ok()
                .and_then(|u| u.domain().map(|d| d.to_string()))
                .unwrap_or_default();
            citation.url = citation_content.clone();
            citation.label(id!(domain)).set_text(cx, &domain);

            while citation.draw(cx, scope).step().is_some() {}
        }
        cx.end_turtle();
        DrawStep::done()
    }
}

impl Citations {
    fn update_citations(&mut self, cx: &mut Cx, citations: &Vec<String>) {
        self.visible = true;
        // compare the vecs, if they are the same, return
        if self.citations.len() == citations.len() {
            let is_same = self
                .citations
                .iter()
                .zip(citations.iter())
                .all(|(a, b)| a == b);
            if is_same {
                return;
            }
        }

        self.citations = citations.clone();
        self.visible = true;
        self.link_preview_children.clear();
        self.loaded_image_indices.clear();
        self.image_blobs.clear();

        for (index, citation) in citations.iter().enumerate() {
            let new_citation = LinkPreviewUI::new_from_ptr(cx, self.citation_template);
            self.link_preview_children.insert(index, new_citation);

            let citation_clone = citation.clone();
            let index_clone = index;
            let ui = self.ui_runner();
        }

        self.redraw(cx);
    }
}

impl CitationsRef {
    pub fn set_citations(&mut self, cx: &mut Cx, citations: &Vec<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.update_citations(cx, citations);
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct LinkPreviewUI {
    #[deref]
    view: View,

    #[rust]
    url: String,
}

impl Widget for LinkPreviewUI {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        if let Some(item) = event.actions().find_widget_action(self.view.widget_uid()) {
            if let ViewAction::FingerUp(fu) = item.cast() {
                if fu.was_tap() {
                    let _ = robius_open::Uri::new(&self.url).open();
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

async fn fetch_image_blob(url: &str) -> Result<Vec<u8>, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        // Trick the server into thinking we're a browser
        .header(USER_AGENT, HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"
        ))
        .send()
        .await?;

    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

fn is_jpeg(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xD8
}

fn is_png(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[0] == 0x89 && bytes[1] == 0x50 && bytes[2] == 0x4E && bytes[3] == 0x47
}
