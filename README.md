# line-server
## Overview 
This is a simple websocket server implemented in the rust programming language. Its job is to serve up a text file accessible over a basic API. 


## API
GET nnnn
If nnnn is a valid line number for the given text file, return "OK\r\n"
and then the nnnn-th line of the specified text file.If nnnn is not a valid line number for the given text file, return
"ERR\r\n".
The first line of the file is line 1 (not line 0).

QUIT
Disconnect client

SHUTDOWN
Shutdown the server

## Port 
The webserver runs on port 10497

## Details
There is no limit on the number of client connections that can be made to the system. The only limitation is that of the hardware running the server. 

In testing this is not the most performant webserver but is decent enough for small to medium use. 

## Building
This server can be built with the [build.sh](build.sh) file included in this directory. 

Internally this will run the cargo build command so an internet connection and installation of rust is needed. 

This system was tested to build and run on Ubuntu 22.04 with 16 threads and 20GB as well as a VM running in a local cloud with 8GB memory two threads and 100 GB drive. 

`./build.sh`

## Running
To run the server you may use the included [run.sh](run.sh) script. This script also calls the above build script to ensure it is the most up to date version of the server. 

You need to pass in a file you would like to host. 

`./run.sh files/test_file.txt`

This should produce output similar to below
```
Building executable
    Finished `release` profile [optimized] target(s) in 0.10s
Running executable
    Finished `release` profile [optimized] target(s) in 0.10s
     Running `target/release/line-server files/test_file.txt`
[2024-06-12 12:30:13.615] [logger] [warn] LOGGING TO PATH: "/home/christian/Documents/rust-projects/line-server/target/release/logs/transcript.log"
[2024-06-12 12:30:13.615] [logger] [info] Loading file: files/test_file.txt
[2024-06-12 12:30:13.620] [logger] [info] Setting up Database....
[🌑                                                                                                                                                      ] 4.00/? [00:00:00] (1.80K it/s)
[2024-06-12 12:30:13.623] [logger] [warn] Database build time: 7.76ms
[2024-06-12 12:30:13.623] [logger] [info] Serving on: localhost:10497

```

Notice this informs you the server is being rebuilt if changes to source are detected, it is telling you where it will be saving its transcript log, show you a progress bar of the DB being populated, and tells you what address you can access the server. 

<b>Note: If no file is passed in the server will default to the [test_file.txt](files/test_file.txt) in the files directory</b>

## Transcript logs
The line server will both log to console as well as save all events to the transcript log noted above. This file will contain details about clients connected, their uid, the command received and sent, as well as response times. 


## Metrics

### Test files
Mulitple test files were used in the evaluation of this project.
Some from https://corpus.canterbury.ac.nz/details/large/ and others of varying size generated with the yes command
EX. `yes hello test file | head -c 500MB > half-gib-file`

### Evaluation 

Benchmarking the server did prove challenging. In order to get some idea of how it handles load two methods were used. 
The first was a delta time log to the transcript file. This reports how long the server took to respond with the line from a `GET n` command. 

The other was using a system called [Artillery](https://www.artillery.io/docs/reference/engines/websocket) an open source web benchmarking tool. This has limited websocket support however. A basic [test file](benchmarking/artillery.yaml) was pulled from their examples. This produces a result file that can be viewed in any web browser here [results](benchmarking/report.html)

This showed the following results 
| Simultaneous Requests / sec    | Response time |
| -------- | ------- |
| 0-10  | ~ 1ms    |
| 10 -20 | ~ 1-5ms|
| 1000    | ~ 1-5sec    |

<b>Note: tested with ulimit at 100000 </b>

### File preprocessing
As part of the server initialization the text file provided is loaded into an in memory db. The goal was to optimize query time across multiple clients. This does require upfront load time however and this is definitely an under performing section of the server. If needed this could be sped up with some multithreading to allow sections of the file to be loaded into the db simultaneously. 

Results of loading various file sizes are listed below

| File Size    | Load time |
| -------- | ------- |
| 43 bytes  | 8ms    |
| 4.0MB | 7.46 sec|
| 500MB    | > 10 min    |
| 1G    | > 10 min    |
| 10G    | > 10 min    |

<b>Note: Above 500MB these numbers are assumed as processing was halted post the 10 min mark</b>

This is clearly a scalability problem that I am not happy with but I have chosen to leave it do to the scope of the project.


## Libraries Used 
All libraries used are open source and available on crates.io

| Library    | Description |
| -------- | ------- |
| [tokio-tungstenite](https://crates.io/crates/tokio-tungstenite)  | Websocket server framework    |
| [duckdb](https://crates.io/crates/duckdb) | An in memory db alternative to sqlite, optimized for reads|
| [zzz](https://crates.io/crates/zzz)    | Rust cli progress bar    |
| [spdlog-rs](https://crates.io/crates/spdlog-rs)    | Rust implementation of spdlog, a performant logging system   |

All of the aboves documentation was consulted as well as the [Rust Book](https://doc.rust-lang.org/book/title-page.html), [Rust by Example](https://doc.rust-lang.org/rust-by-example/index.html), and the [Cargo book](https://doc.rust-lang.org/stable/cargo/). As well as general google search. Any snippets of code pulled were no more than a line or two and from either the above library example files or the rust book code blocks. 

## Time Spent
As this was my first real rust project this took be about a week to do. This included redoing some basic rust projects from their book as a refresher and attempting to deeper understand how to use the efficiencies of rust properly.
