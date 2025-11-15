use sqlite::State;

#[derive(Debug)]
pub struct Entry {
    pub word: String,
    pub wordtype: String,
    pub defs: Vec<String>,
}

pub fn fetch_words(search_q: Option<&str>) -> Result<Vec<Entry>, sqlite::Error> {
    let connection = sqlite::open("resources/database/dictionary.db")?;

    let query = match search_q {
        Some(q) => format!(
            "SELECT word, wordtype FROM entries WHERE word LIKE '{}%' GROUP BY word ORDER BY word LIMIT 10",
            q
        ),
        None => {
            format!("SELECT word, wordtype FROM entries GROUP BY word ORDER BY RANDOM() LIMIT 3")
        }
    };
    let mut statement = connection.prepare(query)?;
    let mut entries = vec![];
    while let Ok(State::Row) = statement.next() {
        let word = statement.read::<String, _>("word").unwrap_or_default();
        let wordtype = statement.read::<String, _>("wordtype").unwrap_or_default();
        let defs = fetch_word_definitions(&connection, &word).unwrap_or_default();

        entries.push(Entry {
            word: word,
            wordtype: wordtype,
            defs: defs,
        });
    }

    Ok(entries)
}

fn fetch_word_definitions(
    connection: &sqlite::Connection,
    word: &str,
) -> Result<Vec<String>, sqlite::Error> {
    let query = format!("SELECT definition FROM entries WHERE word = '{}'", word);
    let mut statement = connection.prepare(query)?;
    let mut definitions = vec![];
    while let Ok(State::Row) = statement.next() {
        definitions.push(
            statement
                .read::<String, _>("definition")
                .unwrap_or_default()
                .replace('\n', "")
                .replace('\r', "")
                .replace("   ", " "),
        );
    }

    Ok(definitions)
}
