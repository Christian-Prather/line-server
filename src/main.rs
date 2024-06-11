/**
 * TODO:
 * (DONE) Argument parse file to load
 * (DONE) Add print out for parsing and starting server so user knows what is happening
 * (DONE) Clean up unused code
 * (DONE?) Add Doc comments
 * (DONE) Only print some things in debug mode
 * (DONE?) Generate docs
 * Add unit tests
 * (DONE?) Handle all errors from instructions
 * (DONE) Split into multiple files
 * Run on VM
 * Evaluate performance
 * Update README with docs
 * (DONE) Add Logging metrics for when server is down and up
 * (DONE) Handle no file passed in
 * (DONE) Add progress bar for DB seeding
 * Re read instructions make sure nothing is missed
 * (DONE) Build release optimized version
 * Add all libraries in README, talk about debug build and relaes with different print outs
 */
//

// Logging
mod logging;
use crate::{
    database::{build_database, get_line},
    logging::logger_setup,
};
use spdlog::prelude::*;
use std::env;

use std::time::Instant;

// Websocket
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{Error as TError, Message, Result};

// DuckDB rust c wrapper. Used as effiecnt SQL DB, optimized for parallel query transactions
// https://duckdb.org/docs/api/rust.html
mod database;
use duckdb::Connection;

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
                    let now = Instant::now();
                    // TODO: clp handle the error if no page number is provided
                    let valid_number = command[1].parse::<i32>().is_ok();
                    if valid_number {
                        // Get the specific line asked for from the db
                        let line_result = get_line(command[1].parse::<i32>().unwrap(), &db);
                        match line_result {
                            Ok(line) => {
                                // Valid line
                                // Send OK response to client informing them the request was received
                                let mut send_message = Message::Text("OK".to_string());
                                write.send(send_message).await?;
                                info!("Server <= OK");

                                // This is definetly bad but fought the borrow on the log for too long
                                let data = line.data;
                                send_message = Message::Text(data.clone());
                                // Send the line back to the client
                                write.send(send_message).await?;
                                info!("Server <= {}", data);
                            }
                            Err(e) => {
                                // Invalid line parse
                                error!("ERR {}", e);
                                let send_message = Message::Text("ERR".to_string());
                                write.send(send_message).await?;
                            }
                        };
                        let elapsed = now.elapsed();
                        warn!(
                            "Elapsed time for {}: {:.2?}",
                            msg.to_text().unwrap(),
                            elapsed
                        );
                    } else {
                        // This is a duplication block from the match above, not sure best way to do this
                        // Invalid line parse
                        error!("ERR {} is not a valid number", command[1]);
                        let send_message = Message::Text("ERR".to_string());
                        write.send(send_message).await?;
                    }
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

    // Websocket
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
