use duckdb::arrow::util::pretty::print_batches;
use duckdb::{params, Connection, Result as dbResult};

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

// use std::{env, io::Error};

use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error as TError, Message, Result},
};

use iron::prelude::*;
use iron::status;
// use router::Router;
use futures_util::{future, stream::SplitSink, SinkExt, StreamExt, TryStreamExt};
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
fn build_database() -> dbResult<Connection> {
    let conn = Connection::open_in_memory()?;

    conn.execute_batch(
        r"CREATE SEQUENCE seq;
          CREATE TABLE document (
                  id              INTEGER PRIMARY KEY DEFAULT NEXTVAL('seq'),
                  data            TEXT NOT NULL,
                  );
        ",
    )?;

    let lines = read_lines("test_file.txt");
    for line in lines {
        println!("{}", line);
        conn.execute("INSERT INTO document (data) VALUES (?)", params![line])?;
    }

    // Uncomment for seeing if file was added to DB correctly
    let mut stmt = conn.prepare("SELECT id, data FROM document")?;
    let line_iter = stmt.query_map([], |row| {
        Ok(Line {
            id: row.get(0)?,
            data: row.get(1)?,
        })
    })?;

    for line in line_iter {
        let l = line.unwrap();
        println!("ID: {}", l.id);
        println!("Found line {:?}", l.data);
    }

    Ok(conn)
}

/// Function responsible for preprocessing the file and loading it into the duckdb database
// fn make_database() {
//     // TODO: Wrap in timing metrics.
//     // Does rust have something built in for that?
//     // Does rust have a built in profiler / tracer as well?
//     let database = build_database().unwrap();
// }

/// Function responsible for getting a specific line from the database
// fn get_line(request: &mut Request) -> IronResult<Response> {
//     let mut response = Response::new();
//     response.set_mut(status::Ok);
//     // response.set_mut(format!("Line is: {}\n", line));

//     Ok(response)
// }

// fn setup_server() {
//     // let mut router = Router::new();
//     // router.get("/", get_line, "root");
//     // // router.("/gcd", post_gcd, "gcd");

//     // println!("Serving on http://localhost:3000.");
//     // Iron::new(router).http("localhost:3000.").unwrap();
// }

fn get_line(line: i32, conn: &Connection) -> dbResult<Line> {
    let text: String = conn.query_row(
        "SELECT data FROM document WHERE id = (?)",
        params![line],
        |row| row.get(0),
    )?;

    println!("TEXT: {}", text);

    let data = Line {
        id: line,
        data: text.to_string(),
    };

    Ok(data)

    // for line in line_iter {
    //     let l = line.unwrap();
    //     println!("ID: {}", l.id);
    //     println!("Found line {:?}", l.data);
    // }
}

async fn accept_connection(stream: TcpStream, db: Connection) -> Result<()> {
    let device_id = stream
        .peer_addr()
        .expect("Connection to peer device failed");
    println!("Device id: {}", device_id);

    let ws = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error in websocket connection");
    println!("New socket connection with: {}", device_id);

    let (mut write, mut read) = ws.split();

    while let Some(msg) = read.next().await {
        let msg = msg?;

        if msg.is_text() || msg.is_binary() {
            let mut send_message = Message::Text("".to_string());
            let command: Vec<&str> = msg.to_text().unwrap().split(" ").collect();
            // Check for the API specific features
            match command[0] {
                "GET" => {
                    // TODO: clp handle the error if no page number is provided
                    send_message = Message::Text(format!("Reading {}", command[1]));
                    write.send(send_message).await?;

                    let line = get_line(command[1].parse().expect("Not valid line number"), &db);
                    send_message = Message::Text(line.unwrap().data);
                    write.send(send_message).await?;
                }
                "QUIT" => {
                    send_message = Message::Text("Quiting".to_string());
                    write.send(send_message).await?;
                }
                "SHUTDOWN" => {
                    send_message = Message::Text("Shuting down".to_string());
                    write.send(send_message).await?;
                }
                _ => println!("Else"),
            }

            // write.send(msg).await?;
        }
    }

    Ok(())

    // let mut msg = "";
    // read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
    //     .forward(write)
    //     .await
    //     .expect("Failed to forward messages")
}

#[tokio::main]
async fn main() -> Result<(), TError> {
    // try_clone()
    let db = build_database().unwrap();

    let test = get_line(2, &db);

    println!("TEST: {}", test.unwrap().data);

    let address = "localhost:10497";
    let socket = TcpListener::bind(&address).await;
    let listener = socket.expect("Failed to connect");
    println!("Serving on: {}", address);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(
            stream,
            db.try_clone()
                .expect("Cant clone db connection, you're silly"),
        ));
    }

    Ok(())

    // let mut router = Router::new();
    // router.get("/:line", get_line, "line");
    // // router.("/gcd", post_gcd, "gcd");

    // println!("Serving on http://localhost:10497");
    // Iron::new(router).http("localhost:10497").unwrap();
}
