use sqlite::State;
use std::path::PathBuf;

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

#[must_use]
pub fn search_query(search_q: Option<&str>, limit: Option<u8>) -> String {
    let limit = limit.unwrap_or(10);
    match search_q {
        Some(q) => format!(
            "SELECT id, word FROM words WHERE word LIKE '{}%' ESCAPE '\\' ORDER BY CASE WHEN word NOT LIKE '% %' THEN 0 ELSE 1 END, LOWER(word) ASC LIMIT {limit}",
            escape_like(q)
        ),
        None => format!("SELECT id, word FROM words ORDER BY RANDOM() LIMIT {limit}"),
    }
}

pub fn fetch_words(query: &str) -> Result<Vec<Entry>, sqlite::Error> {
    let connection = sqlite::open(get_dictionary_path())?;
    let mut statement = connection.prepare(query)?;
    let mut entries = vec![];
    while let Ok(State::Row) = statement.next() {
        let id = statement.read::<String, _>("id").unwrap_or_default();
        let word = statement.read::<String, _>("word").unwrap_or_default();
        let defs = fetch_word_definitions(&connection, &id).unwrap_or_default();

        entries.push(Entry { id, word, defs });
    }

    Ok(entries)
}

fn fetch_word_definitions(
    connection: &sqlite::Connection,
    word_id: &str,
) -> Result<Vec<Definition>, sqlite::Error> {
    let query = format!(
        "SELECT def, speech_part, example FROM definitions WHERE word_id = '{}' GROUP BY (id)",
        word_id
    );
    let mut statement = connection.prepare(query)?;
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
        .replace('\'', "''")
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
