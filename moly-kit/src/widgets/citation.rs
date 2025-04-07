use crate::utils::asynchronous::spawn;
use crate::utils::scraping::*;
use makepad_widgets::*;
use url::Url;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    BOLD_FONT = {
        font: {path: dep("crate://makepad-widgets/resources/IBMPlexSans-SemiBold.ttf")}
    }

    pub Citation = {{Citation}} <RoundedView> {
        flow: Down,
        height: Fit,
        cursor: Hand,
        width: 200,
        padding: 6,
        spacing: 4,
        draw_bg: {
            color: #f2f2f2
            border_radius: 3
        }

        <View> {
            height: Fit,
            align: {y: 0.5},
            icon = <Image> {
                width: 16,
                height: 16,
                source: dep("crate://self/resources/link.png")
            }

            site = <Label> {
                draw_text: {
                    text_style: <BOLD_FONT>{font_size: 9},
                    color: #555,
                }
            }
        }

        title = <Label> {
            draw_text: {
                text_style: {font_size: 8.5},
                color: #000,
            }
        }
    }
}

#[derive(Debug, Clone, DefaultNone)]
pub enum CitationAction {
    Open(String),
    None,
}

#[derive(Live, Widget, LiveHook)]
pub struct Citation {
    #[deref]
    deref: View,

    #[rust]
    url: Option<String>,
}

impl Widget for Citation {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);

        if let Hit::FingerUp(fu) = event.hits(cx, self.area()) {
            if fu.was_tap() {
                if let Some(url) = &self.url {
                    cx.widget_action(
                        self.widget_uid(),
                        &scope.path,
                        CitationAction::Open(url.clone()),
                    );
                }
            }
        }
    }
}

impl Citation {
    pub fn set_url_once(&mut self, cx: &mut Cx, url: String) {
        if self.url.is_some() {
            return;
        }

        self.set_url(cx, url);
    }

    fn set_url(&mut self, cx: &mut Cx, url: String) {
        self.url = Some(url);

        // Step 1 is to set texts to something that will not fail.
        self.set_initial_info(cx);

        // Step 2 is to try refining the texts if the URL is valid, without fetching any data.
        let Ok(()) = self.try_refine_info(cx) else {
            return;
        };

        // Step 3 is to try fetching actual title and favicon from the internet and
        // use that. This is async and has a delay. Not possible if step 2 failed.
        self.try_fetch_info(cx);
    }

    fn set_initial_info(&mut self, cx: &mut Cx) {
        let site = self.label(id!(site));
        let title = self.label(id!(title));
        let url = self.url.as_deref().unwrap();

        site.set_text(cx, url);
        title.set_text(cx, url);
    }

    fn try_refine_info(&mut self, cx: &mut Cx) -> Result<(), ()> {
        let site = self.label(id!(site));
        let title = self.label(id!(title));
        let url = self.url.as_deref().unwrap();

        let url = Url::parse(url).map_err(|_| ())?;
        let host = url.host_str().ok_or(())?;
        let path = url.path();

        site.set_text(cx, host);
        title.set_text(cx, path);
        Ok(())
    }

    fn try_fetch_info(&mut self, _cx: &mut Cx) {
        let url = self.url.clone().unwrap();
        let ui = self.ui_runner();
        spawn(async move {
            let Ok(document) = fetch_html(&url).await else {
                return;
            };

            if let Some(title) = extract_title(&document) {
                ui.defer_with_redraw(move |me, cx, _| {
                    me.label(id!(title)).set_text(cx, &title);
                });
            }

            // TODO: Extract favicon
            // TODO: Support .ico and .svg.
            // TODO: Support relative and data urls.
            // TODO: Support jpg, png, and gif (less common favicon types).
        });
    }
}
