// use duckdb::arrow::util::pretty::print_batches;
// use duckdb::{params, Connection, Result};

// Web app section
// Based on Hyper, was going to use that until I realized it was used in iron
extern crate iron;
extern crate logger;
extern crate router;

extern crate urlencoded;
// use std::net::{TcpListener, TcpStream};
// use std::str::FromStr;
// use urlencoded::UrlEncodedBody;

use std::fs::read_to_string;

use std::{env, io::Error};

use iron::prelude::*;
use iron::status;
// use router::Router;
use futures_util::{future, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug)]
struct Line {
    id: i32,
    data: String,
}

// TODO use the more efficient method described in the rust docs
fn read_lines(filename: &str) -> Vec<String> {
    let mut result = Vec::new();
    for line in read_to_string(filename).unwrap().lines() {
        result.push(line.to_string());
    }
    result
}

/// Function responsible for building the database
// fn build_database() -> Result<()> {
//     let conn = Connection::open_in_memory()?;

//     conn.execute_batch(
//         r"CREATE SEQUENCE seq;
//           CREATE TABLE line (
//                   id              INTEGER PRIMARY KEY DEFAULT NEXTVAL('seq'),
//                   data            TEXT NOT NULL,
//                   );
//         ",
//     )?;

//     let lines = read_lines("test_file.txt");
//     for line in lines {
//         println!("{}", line);
//         conn.execute("INSERT INTO line (data) VALUES (?)", params![line])?;
//     }

//     let mut stmt = conn.prepare("SELECT id, data FROM line")?;
//     let line_iter = stmt.query_map([], |row| {
//         Ok(Line {
//             id: row.get(0)?,
//             data: row.get(1)?,
//         })
//     })?;

//     for line in line_iter {
//         let l = line.unwrap();
//         println!("ID: {}", l.id);
//         println!("Found line {:?}", l.data);
//     }

//     Ok(())
// }

/// Function responsible for preprocessing the file and loading it into the duckdb database
// fn preprocess_file() {
//     // TODO: Wrap in timing metrics.
//     // Does rust have something built in for that?
//     // Does rust have a built in profiler / tracer as well?
//     let _ = build_database();
// }

/// Function responsible for getting a specific line from the database
fn get_line(request: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();
    response.set_mut(status::Ok);
    // response.set_mut(format!("Line is: {}\n", line));

    Ok(response)
}

fn setup_server() {
    // let mut router = Router::new();
    // router.get("/", get_line, "root");
    // // router.("/gcd", post_gcd, "gcd");

    // println!("Serving on http://localhost:3000.");
    // Iron::new(router).http("localhost:3000.").unwrap();
}

async fn accept_connection(stream: TcpStream) {
    let device_id = stream
        .peer_addr()
        .expect("Connection to peer device failed");
    println!("Device id: {}", device_id);

    let ws = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error in websocket connection");
    println!("New socket connection with: {}", device_id);

    let (write, read) = ws.split();

    let mut msg = "";
    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("Failed to forward messages")
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // preprocess_file();
    let address = "localhost:10497";
    let socket = TcpListener::bind(&address).await;
    let listener = socket.expect("Failed to connect");
    println!("Serving on: {}", address);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())

    // setup_server();
    // let mut router = Router::new();
    // router.get("/:line", get_line, "line");
    // // router.("/gcd", post_gcd, "gcd");

    // println!("Serving on http://localhost:10497");
    // Iron::new(router).http("localhost:10497").unwrap();
}
