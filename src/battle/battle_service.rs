use makepad_widgets::{Actions, Cx, DefaultNone};

use super::battle_sheet::Sheet;

/// Isolated interface to connect and work with the remote battle server.
pub struct BattleService {
    /// Identify this instance to handle responses in isolation.
    /// `Cx::post_action` by itself is global.
    id: usize,
}

impl BattleService {
    /// Create a new identified instance.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        Self { id }
    }

    pub fn download_battle_sheet(&self, code: String) {
        let id = self.id;
        std::thread::spawn(move || {
            // let response = reqwest get json...
            std::thread::sleep(std::time::Duration::from_secs(3));

            let text = include_str!("battle_sheet.json");
            match serde_json::from_str::<Sheet>(text) {
                Ok(mut sheet) => {
                    sheet.code = code;
                    Cx::post_action((id, Response::BattleSheetDownloaded(sheet)));
                }
                Err(err) => Cx::post_action((id, Response::Error(err.to_string()))),
            };
        });
    }

    pub fn send_battle_sheet(&self, _sheet: Sheet) {
        let id = self.id;
        std::thread::spawn(move || {
            // let response = reqwest post json...
            std::thread::sleep(std::time::Duration::from_secs(3));

            Cx::post_action((id, Response::BattleSheetSent));
        });
    }

    pub fn battle_sheet_downloaded<'a>(&'a self, actions: &'a Actions) -> Option<&'a Sheet> {
        self.responses(actions)
            .filter_map(|response| match response {
                Response::BattleSheetDownloaded(sheet) => Some(sheet),
                _ => None,
            })
            .next()
    }

    pub fn battle_sheet_sent(&self, actions: &Actions) -> bool {
        self.responses(actions)
            .any(|response| matches!(response, Response::BattleSheetSent))
    }

    pub fn failed<'a>(&'a self, actions: &'a Actions) -> Option<&'a str> {
        self.responses(actions)
            .filter_map(|response| match response {
                Response::Error(err) => Some(err.as_str()),
                _ => None,
            })
            .next()
    }

    /// Handle responses sent from this specific instance.
    fn responses<'a>(&'a self, actions: &'a Actions) -> impl Iterator<Item = &'a Response> {
        actions
            .iter()
            .filter_map(move |action| action.downcast_ref::<(usize, Response)>())
            .filter(|(id, _)| *id == self.id)
            .map(|(_, response)| response)
    }
}

/// Actions sent from other threads thru `Cx::post_action` representing async responses.
///
/// Doesn't actually need private, nor the `responses` function, but just exposing
/// event handling thru methods like button's `clicked` is less error prone and more
/// elegant, so would be ideal to not use this from outside.
#[derive(Debug, Clone, DefaultNone)]
enum Response {
    BattleSheetDownloaded(Sheet),
    BattleSheetSent,
    Error(String),
    None,
}
