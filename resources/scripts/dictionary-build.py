import json
import sqlite3

def main() -> None:
    
    with open('../../wordset-dictionary/allwords_wordset.json/aacompletewordset.json', 'r') as file:
        con = sqlite3.connect("../database/wordset.db")
        cur = con.cursor()

        cur.execute("DELETE FROM words")
        con.commit()
        cur.execute("DELETE FROM definitions")
        con.commit()

        data = json.load(file)
        for word in data:

            query = "INSERT INTO words (id, word) VALUES (?, ?)"
            word_insert = (data[word]["wordset_id"], data[word]["word"])
            cur.execute(query, word_insert)
            con.commit()

            if "meanings" in data[word]:
                for meaning in data[word]["meanings"]:

                    query = "INSERT INTO definitions (id, word_id, speech_part, def, example) VALUES (?, ?, ?, ?, ?)"
                    def_insert = (meaning["id"], data[word]["wordset_id"], meaning["speech_part"], meaning["def"], meaning["example"] if "example" in meaning else "")
                    cur.execute(query, def_insert)
                    con.commit()

main()