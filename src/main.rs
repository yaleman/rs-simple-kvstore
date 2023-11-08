use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener};
use std::{thread, time};

const SLEEP_TIMER: time::Duration = time::Duration::from_millis(50);

fn process_loop() {
    let mut data_items: HashMap<String, String> = HashMap::new();
    let mut client_connections = Vec::new();
    let addr = "127.0.0.1:12001";
    println!("Starting up on {}", &addr);
    let listener = TcpListener::bind(addr).expect("could not bind to port");

    if listener.set_nonblocking(true).is_err() {
        println!("Cannot set server to non blocking!");
        return;
    }

    loop {
        match listener.accept() {
            Ok((socket, addr)) => {
                println!("new client connection={addr:?}");
                if socket.set_nonblocking(true).is_err() {
                    println!("could not set client to non blocking");
                    return;
                }
                client_connections.push(socket);
            }
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => {}
                _ => println!("Unexpected error on server socket {e:?}"),
            },
        }
        let mut next_connections = Vec::new();
        for mut conn in client_connections.into_iter() {
            let mut request = String::new();
            match conn.read_to_string(&mut request) {
                Ok(_) => {}
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock => { /*read_to_string doesn't fit well with would block*/
                    }
                    ErrorKind::ConnectionReset => {
                        println!("Client connection reset");
                    }
                    _ => println!("Got err {e:?}"),
                },
            }
            if request == "shutdown" {
                return;
            } else if request == "close" {
                if let Err(err) = conn.shutdown(Shutdown::Both) {
                    println!("Failed to shutdown client connection: {:?}", err);
                    continue;
                }
            } else if request.is_empty() {
                next_connections.push(conn);
                continue;
            }
            let req_parts: Vec<&str> = request.split('=').collect();
            match req_parts.len() {
                1 => {
                    let val = data_items
                        .get(req_parts[0].trim())
                        .map(|s| s.to_owned())
                        .unwrap_or("NONE".to_string());
                    if let Err(err) = conn.write_all(val.as_bytes()) {
                        println!("Write Problem: {:?}", err);
                        continue;
                    }
                }
                2 => {
                    println!("Setting {} to {}", req_parts[0].trim(), req_parts[1].trim());
                    data_items.insert(
                        req_parts[0].trim().to_string(),
                        req_parts[1].trim().to_string(),
                    );
                    if let Err(err) = conn.write_all("PUT".as_bytes()) {
                        println!("Write Problem: {:?}", err);
                        continue;
                    }
                }
                _ => println!("Unknown request format {req_parts:?}"),
            }
            if let Err(err) = conn.flush() {
                println!("Flush Problem: {:?}", err);
                continue;
            }
            next_connections.push(conn)
        }
        client_connections = next_connections;
        thread::sleep(SLEEP_TIMER);
    }
}

fn main() {
    process_loop();

    println!("Shutting down!");
}
