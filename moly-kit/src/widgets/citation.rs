use crate::utils::asynchronous::spawn;
use crate::utils::scraping::*;
use makepad_widgets::*;
use url::Url;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub Citation = {{Citation}} <RoundedView> {
        flow: Down,
        height: Fit,
        width: 200,
        padding: 6,
        spacing: 4,
        draw_bg: {
            color: #f0f0f0
            radius: 3
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
                    color: #555,
                }
            }
        }

        <View> {
            height: Fit,
            title = <Label> {
                draw_text: {
                    color: #000,
                }
            }
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct Citation {
    #[deref]
    deref: View,

    #[rust]
    set: bool,
}

impl Widget for Citation {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope)
    }
}

impl Citation {
    pub fn set_url_once(&mut self, cx: &mut Cx, url: &str) {
        if self.set {
            return;
        }

        self.set = true;
        self.set_url(cx, url);
    }

    fn set_url(&mut self, cx: &mut Cx, url: &str) {
        // Step 1 is to set texts to something that will not fail.
        self.set_initial_info(cx, url);

        // Step 2 is to try refining the texts if the URL is valid, without fetching any data.
        let Ok(()) = self.try_refine_info(cx, url) else {
            return;
        };

        // Step 3 is to try fetching actual title and favicon from the internet and
        // use that. This is async and has a delay. Not possible if step 2 failed.
        self.try_fetch_info(cx, url.to_string());
    }

    fn set_initial_info(&mut self, cx: &mut Cx, url: &str) {
        self.set_texts(cx, url, url);
    }

    fn try_refine_info(&mut self, cx: &mut Cx, url: &str) -> Result<(), ()> {
        let url = Url::parse(url).map_err(|_| ())?;
        let host = url.host_str().ok_or(())?;
        let path = url.path();

        self.set_texts(cx, host, path);
        Ok(())
    }

    fn set_texts(&mut self, cx: &mut Cx, site: &str, title: &str) {
        self.label(id!(site)).set_text(cx, site);
        self.label(id!(title)).set_text(cx, title);
    }

    fn try_fetch_info(&mut self, _cx: &mut Cx, url: String) {
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
