use duckdb::{params, Connection, Result as dbResult};

// Convert data to string
use spdlog::prelude::*;
use std::fs::read_to_string;
use zzz::{ProgressBar, ProgressBarIterExt as _};

use std::time::Instant;

/// In program representation of row entry in db
#[derive(Debug)]
pub struct Line {
    /// Line of text that is in the db
    pub data: String,
}

// TODO This is very limited at scale, since a db  is ACID compliant a good speed up would
// be to split this into threads

/// Reads all the lines of the txt file and loads them in a vector for adding to db
/// Its not lost on me that this vector would serve our purpose rather than use a db
/// but as a db would allow more scalability and I wanted to try duckdb here we are...
fn load_lines(filename: &str, conn: &Connection) {
    // Iterate over all the lines and add them to the db
    for line in read_to_string(filename)
        .unwrap()
        .lines()
        .with_progress(ProgressBar::smart())
    {
        // Insert line into the db
        conn.execute("INSERT INTO document (data) VALUES (?)", params![line])
            .expect("Error populating DB");
    }
}

/// Function used in debugging to printing out db (COMMENTED OUT ON PURPOSE)
// pub fn print_database_lines(conn: &Connection) -> dbResult<()> {
//     // Debug helper for seeing if file was added to DB correctly
//     let mut stmt = conn.prepare("SELECT id, data FROM document")?;
//     let line_iter = stmt.query_map([], |row| Ok(Line { data: row.get(1)? }))?;

//     for line in line_iter {
//         let l = line.unwrap();
//         println!("Found line {:?}", l.data);
//     }
//     Ok(())
// }

/// Get a single line from the db based on row number (id field)
pub fn get_line(line: i32, conn: &Connection) -> dbResult<Line> {
    // SQL query
    let text: String = conn.query_row(
        "SELECT data FROM document WHERE id = (?)",
        params![line],
        |row| row.get(0),
    )?;

    // Save data to Line object for use in other functions
    let data = Line {
        data: text.to_string(),
    };

    Ok(data)
}

/// Function responsible for building the database. This can take a minute if source file is large but makes query very fast and parallel.
pub fn build_database(file: &String) -> dbResult<Connection> {
    let now = Instant::now();

    // Create a db connection in ram. This is not persistent but if wanted can use disk instead.
    let conn = Connection::open_in_memory()?;

    // Make a table in db called document that has unique id (row number) and the line of text
    conn.execute_batch(
        r"CREATE SEQUENCE seq;
          CREATE TABLE document (
                  id              INTEGER PRIMARY KEY DEFAULT NEXTVAL('seq'),
                  data            TEXT NOT NULL,
                  );
        ",
    )?;

    // Inform the user that the database is being setup.
    info!("Setting up Database....");
    // Get all the lines from the text file
    load_lines(file, &conn);
    // Iterate over each line and add it to the db. Progress bar used to indicate system is not hung.
    // for line in lines.into_iter().with_progress(ProgressBar::smart()) {
    //     conn.execute("INSERT INTO document (data) VALUES (?)", params![line])?;
    // }
    let elapsed = now.elapsed();
    warn!("Database build time: {:.2?}", elapsed);
    // Return db connection
    Ok(conn)
}
