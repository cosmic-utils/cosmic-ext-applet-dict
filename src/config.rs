// SPDX-License-Identifier: GPL-3.0-only

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    pub bookmarks: Vec<String>,
}

impl Config {
    pub fn is_bookmarked(&self, word: &str) -> bool {
        self.bookmarks.iter().any(|bookmark| bookmark == word)
    }

    pub fn toggle_bookmark(&mut self, word: &str) {
        if let Some(index) = self.bookmarks.iter().position(|bookmark| bookmark == word) {
            self.bookmarks.remove(index);
        } else {
            self.bookmarks.push(word.to_owned());
        }
    }
}
