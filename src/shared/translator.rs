use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::error::Error;
use makepad_live_compiler::live_registry;
use makepad_widgets::*;

#[derive(Debug, Clone)]
pub struct Translator {
    current_language: String,
    translations: HashMap<String, LiveFileId>,
}

impl Translator {
    pub fn new(default_lang: &str) -> Self {
        Translator {
            current_language: default_lang.to_string(),
            translations: HashMap::new(),
        }
    }

    pub fn set_translations(&mut self, cx: &Cx, translations: &[(&str, &str)]) -> Result<(), Box<dyn Error>> {
        let mut lang_map = HashMap::new();
        
        for (lang, file_name) in translations {
            // Load translations from live design
            let live_registry_ref = cx.live_registry.borrow();
            let file_id = live_registry_ref.file_name_to_file_id(file_name)
                .ok_or_else(|| format!("Failed to find file: {}", file_name))?;
            
            lang_map.insert(lang.to_string(), file_id);
        }

        if !lang_map.contains_key(&self.current_language) {
            return Err("Default language not found in translations".into());
        }

        self.translations = lang_map;
        Ok(())
    }

    pub fn add_translations(&mut self, lang: &str, file_name: &str, cx: &Cx) -> Result<(), Box<dyn Error>> {
        let live_registry_ref = cx.live_registry.borrow();
        let file_id = live_registry_ref.file_name_to_file_id(file_name)
            .ok_or_else(|| format!("Failed to find file: {}", file_name))?;
        
        self.translations.insert(lang.to_string(), file_id);
        Ok(())
    }

    pub fn tr(&mut self, cx: &Cx, tr_live_id: LiveId) -> Option<String> {
        let file_id = self.translations.get(&self.current_language)?;

        // TODO: take content from the live design
        let live_registry_ref = cx.live_registry.borrow();
        dbg!("HERE");
        let live_file = live_registry_ref.file_id_to_file(*file_id);
        dbg!("HERE");
        // dbg!(&live_file.expanded.nodes);
        let nodes = &live_file.expanded.nodes;

	dbg!(file_id);
	dbg!(live_registry_ref.file_id_to_file_name(*file_id));
        let target = live_registry_ref.find_scope_target(tr_live_id, nodes)?;
        match target {
            live_registry::LiveScopeTarget::LocalPtr(local_ptr) => {
                let live_node = nodes.get(local_ptr)?;
                let value = live_registry_ref.live_node_as_string(live_node);
                value
            }
            live_registry::LiveScopeTarget::LivePtr(live_ptr) => {
                let node = live_registry_ref.ptr_to_node(live_ptr);
                let value = live_registry_ref.live_node_as_string(node);
                value
            }
        }
    }

    pub fn set_language(&mut self, language: &str) -> Result<(), String> {
        if self.translations.contains_key(language) {
            self.current_language = language.to_string();
            Ok(())
        } else {
            Err(format!("Language not available: {}", language))
        }
    }
}
