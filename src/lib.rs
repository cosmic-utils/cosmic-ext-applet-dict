// SPDX-License-Identifier: GPL-3.0-only

use sqlite::{OpenFlags, State};
use std::path::PathBuf;

const DEFAULT_LIMIT: u8 = 10;

#[derive(Debug)]
pub struct Entry {
    pub id: String,
    pub word: String,
    pub defs: Vec<Definition>,
}

#[derive(Debug)]
pub struct Definition {
    pub def: String,
    pub speech_part: String,
    pub example: String,
}

/// Searches for words beginning with `prefix`.
///
/// `limit` defaults to 10 when `None`.
///
/// # Errors
/// Returns an error if the dictionary cannot be opened or the query fails.
pub fn search_words(prefix: &str, limit: Option<u8>) -> Result<Vec<Entry>, sqlite::Error> {
    let connection = open_dictionary()?;
    let mut statement = connection.prepare(
        "SELECT id, word FROM words WHERE word LIKE ?1 || '%' ESCAPE '\\' ORDER BY CASE WHEN word NOT LIKE '% %' THEN 0 ELSE 1 END, LOWER(word) ASC LIMIT ?2",
    )?;
    statement.bind((1, escape_like(prefix).as_str()))?;
    statement.bind((2, i64::from(limit.unwrap_or(DEFAULT_LIMIT))))?;

    Ok(fetch_entries(&connection, &mut statement))
}

// Returns up to `limit` random words.
///
/// `limit` defaults to 10 when `None`.
///
/// # Errors
/// Returns an error if the dictionary cannot be opened or the query fails.
pub fn random_words(limit: Option<u8>) -> Result<Vec<Entry>, sqlite::Error> {
    let connection = open_dictionary()?;
    let mut statement = match limit.unwrap_or(DEFAULT_LIMIT) {
        0 => return Ok(Vec::new()),
        1 => connection.prepare(
            "SELECT id, word FROM words \
             WHERE rowid >= 1 + abs(random() % (SELECT MAX(rowid) FROM words)) \
             ORDER BY rowid LIMIT 1",
        )?,
        limit => {
            let mut statement =
                connection.prepare("SELECT id, word FROM words ORDER BY RANDOM() LIMIT ?1")?;
            statement.bind((1, i64::from(limit)))?;
            statement
        }
    };

    Ok(fetch_entries(&connection, &mut statement))
}

fn open_dictionary() -> Result<sqlite::Connection, sqlite::Error> {
    sqlite::Connection::open_with_flags(get_dictionary_path(), OpenFlags::new().with_read_only())
}

fn fetch_entries(
    connection: &sqlite::Connection,
    statement: &mut sqlite::Statement<'_>,
) -> Vec<Entry> {
    let mut entries = vec![];
    while let Ok(State::Row) = statement.next() {
        let id = statement.read::<String, _>("id").unwrap_or_default();
        let word = statement.read::<String, _>("word").unwrap_or_default();
        let defs = fetch_word_definitions(connection, &id).unwrap_or_default();

        entries.push(Entry { id, word, defs });
    }

    entries
}

fn fetch_word_definitions(
    connection: &sqlite::Connection,
    word_id: &str,
) -> Result<Vec<Definition>, sqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT def, speech_part, example FROM definitions WHERE word_id = ?1 GROUP BY (id)",
    )?;
    statement.bind((1, word_id))?;

    let mut definitions: Vec<Definition> = vec![];
    while let Ok(State::Row) = statement.next() {
        definitions.push(Definition {
            def: statement.read::<String, _>("def").unwrap_or_default(),
            speech_part: statement
                .read::<String, _>("speech_part")
                .unwrap_or_default(),
            example: statement.read::<String, _>("example").unwrap_or_default(),
        });
    }

    Ok(definitions)
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn get_dictionary_path() -> PathBuf {
    const APP_ID: &str = "dev.cappsy.CosmicExtAppletDict";
    const DB_FILENAME: &str = "wordset.db";

    // Are we in dev mode?
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let dev_path = exe_dir.join("../../resources/database/").join(DB_FILENAME);
            if dev_path.exists() {
                return dev_path.canonicalize().unwrap_or(dev_path);
            }
        }
    }

    // Are we in the flatpak?
    if is_flatpak() {
        let flatpak_path = PathBuf::from("/app/share").join(APP_ID).join(DB_FILENAME);
        if flatpak_path.exists() {
            return flatpak_path;
        }
    }
    // Are we running on the host?
    else {
        let data_dirs = std::env::var("XDG_DATA_DIRS")
            .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

        for dir in data_dirs.split(':') {
            let path = PathBuf::from(dir).join(APP_ID).join(DB_FILENAME);

            if path.exists() {
                return path;
            }
        }
    }

    // oh well
    PathBuf::from("/usr/share").join(APP_ID).join(DB_FILENAME)
}

#[cfg(feature = "flatpak")]
fn is_flatpak() -> bool {
    true
}

#[cfg(not(feature = "flatpak"))]
fn is_flatpak() -> bool {
    false
}
