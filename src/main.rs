/**
 * TODO:
 * (DONE) Argument parse file to load
 * (DONE) Add print out for parsing and starting server so user knows what is happening
 * (DONE) Clean up unused code
 * (DONE?) Add Doc comments
 * (DONE) Only print some things in debug mode
 * (DONE?) Generate docs
 * Add unit tests
 * Handle all errors from instructions
 * Split into multiple files
 * Run on VM
 * Evaluate performance
 * Update README with docs
 * (DONE) Add Logging metrics for when server is down and up
 * Handle no file passed in
 * (DONE) Add progress bar for DB seeding
 * Re read instructions make sure nothing is missed
 * (DONE) Build release optimized version
 * Add all libraries in README, talk about debug build and relaes with different print outs
 */
//

// Logger, command line args
use std::{env, path::PathBuf, sync::Arc};

use spdlog::{prelude::*, sink::FileSink, sink::Sink};

// Convert data to string
use std::fs::read_to_string;

// Progress bar for database setup
use zzz::{ProgressBar, ProgressBarIterExt as _};

// Websocket
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{Error as TError, Message, Result};

// DuckDB rust c wrapper. Used as effiecnt SQL DB, optimized for parallel query transactions
// https://duckdb.org/docs/api/rust.html
use duckdb::{params, Connection, Result as dbResult};

/// In program representation of row entry in db
#[derive(Debug)]
struct Line {
    /// Line of text that is in the db
    data: String,
}

// TODO use the more efficient method described in the rust docs
/// Reads all the lines of the txt file and loads them in a vector for adding to db
/// Its not lost on me that this vector would serve our puprose rather than use a db
/// but as a db would allow more scalability and I wanted to try duckdb here we are...
fn read_lines(filename: &str) -> Vec<String> {
    // New vector for the lines, each row in file is an element in the vector
    let mut result = Vec::new();
    // Iterate over all the lines and add them to the vector
    for line in read_to_string(filename).unwrap().lines() {
        result.push(line.to_string());
    }
    // Vector of lines
    result
}

/// Funtion used in debugging to printing out db (COMMENTED OUT ON PURPOSE)
// fn print_database_lines(conn: &Connection) -> dbResult<()> {
//     // Debug helper for seeing if file was added to DB correctly
//     let mut stmt = conn.prepare("SELECT id, data FROM document")?;
//     let line_iter = stmt.query_map([], |row| Ok(Line { data: row.get(1)? }))?;

//     for line in line_iter {
//         let l = line.unwrap();
//         println!("Found line {:?}", l.data);
//     }
//     Ok(())
// }

/// Function responsible for building the database. This can take a minute if source file is large but makes query very fast and parallel.
fn build_database(file: &String) -> dbResult<Connection> {
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
    let lines = read_lines(file);
    // Iterate over each line and add it to the db. Progress bar used to indicate system is not hung.
    for line in lines.into_iter().with_progress(ProgressBar::smart()) {
        conn.execute("INSERT INTO document (data) VALUES (?)", params![line])?;
    }

    // Return db connection
    Ok(conn)
}

/// Get a single line from the db based on row number (id field)
fn get_line(line: i32, conn: &Connection) -> dbResult<Line> {
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

/// Handle a new client connection. Each unique client gets a thread of this function.
async fn accept_connection(stream: TcpStream, db: Connection) -> Result<()> {
    // Get the uid of client device connecting to server
    let device_id = stream
        .peer_addr()
        .expect("Connection to peer device failed");

    //Print the device id
    debug!("Device id: {}", device_id);

    // Set up a websocket connection to client right away
    let ws = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error in websocket connection");

    // Inform system of new connection
    debug!("New socket connection with: {}", device_id);

    // Split the websocket connection into its read and write components
    let (mut write, mut read) = ws.split();

    // Constant loop to read and process incoming messages. This is the implementation of the API endpoints
    while let Some(msg) = read.next().await {
        // Incoming message from client
        let msg = msg?;

        // Is the message of a valid data type
        if msg.is_text() {
            // Convert it to a usable vec of strings split on a space
            let command: Vec<&str> = msg.to_text().unwrap().split(" ").collect();
            // Check for the API specific features
            info!("Client => {}", msg.to_text().unwrap());

            match command[0] {
                // Get a line from the text file
                "GET" => {
                    // TODO: clp handle the error if no page number is provided
                    // Send OK response to client informing them the request was received
                    let mut send_message = Message::Text("OK".to_string());

                    write.send(send_message).await?;
                    info!("Server <= OK");

                    // Get the specific line asked for from the db
                    let line = get_line(command[1].parse().expect("Not valid line number"), &db);
                    // This is definetly bad but fought the borrow on the log for too long
                    let data = line.unwrap().data;
                    send_message = Message::Text(data.clone());
                    // Send the line back to the client
                    write.send(send_message).await?;
                    info!("Server <= {}", data);
                }
                // Disconnect this specific client
                "QUIT" => {
                    info!("<< Server disconnected from client: {} >>", device_id);
                    return Ok(());
                }
                // Shutdown the server completely. Note: any client can call this and all other clients will also be disconnected.
                "SHUTDOWN" => {
                    info!("<< Server disconnected from ALL client >>");
                    std::process::exit(0)
                }
                // Command not valid API endpoint
                _ => {
                    let send_message = Message::Text(
                        "Command not recognized, try GET n, QUIT, or SHUTDOWN".to_string(),
                    );
                    write.send(send_message).await?;
                    warn!(
                        "Command '{}' not recognized, try GET n, QUIT, or SHUTDOWN",
                        command[0]
                    );
                }
            }
        }
    }

    Ok(())
}

/// Setup up spdlogger for console and file logging
fn logger_setup() {
    // Logging lines pulled from spdlog repo example
    const LOG_FILE: &str = "logs/transcript.log";

    let path: PathBuf = env::current_exe().unwrap().parent().unwrap().join(LOG_FILE);

    let file_sink: Arc<FileSink> = Arc::new(
        FileSink::builder()
            .path(&path)
            .truncate(true)
            .build()
            .expect("Failed to make file sink"),
    );

    // let sinks: Vec<Arc<dyn Sink>> = spdlog::default_logger().sinks().to_owned();
    let mut sinks: Vec<Arc<dyn Sink>> = spdlog::default_logger().sinks().to_owned();
    sinks.push(file_sink);

    let mut builder: LoggerBuilder = Logger::builder();
    let builder: &mut LoggerBuilder = builder.sinks(sinks).level_filter(LevelFilter::All);

    let logger: Arc<Logger> = Arc::new(
        builder
            .name("logger")
            .build()
            .expect("Failed to build logger"),
    );
    // Flush when log "warn" and more severe logs.
    logger.set_flush_level_filter(LevelFilter::MoreSevereEqual(Level::Debug));

    spdlog::set_default_logger(logger);
    // Logger
    warn!("LOGGING TO PATH: {:?}", path);
}

/// Main thread of program
#[tokio::main]
async fn main() -> Result<(), TError> {
    // Logger
    logger_setup();

    // Handle argument parsing for file to serve
    let args: Vec<String> = env::args().collect();
    let mut seed_file: &String = &"files/test_file.txt".to_string();
    if args.len() > 1 {
        seed_file = &args[1];
    }

    info!("Loading file: {}", &seed_file);

    // Build the db from the file passed in
    let db = build_database(seed_file).unwrap();

    // Default server address and port. No need to change so not arguments
    let address = "localhost:10497";
    // Make the websocket at the given address
    let socket = TcpListener::bind(&address).await;
    let listener = socket.expect("Failed to connect");
    // Inform user what address server is running on
    info!("Serving on: {}", address);

    // Loop to add new clients as they request connections
    while let Ok((stream, _)) = listener.accept().await {
        // New thread per client connection. Pass in clone of db connection.
        // The db should handle ACID transactions but the db is also used as read only in application
        // so its even more not a concern to share.
        tokio::spawn(accept_connection(
            stream,
            db.try_clone()
                .expect("Cant clone db connection, you're silly"),
        ));
    }

    Ok(())
}
