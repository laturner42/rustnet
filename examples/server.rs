#![feature(old_io)]

extern crate sdl2;
extern crate rustnet;

use sdl2::sdl;

use std::thread;
use std::sync::mpsc;

use rustnet::SocketWrapper as Socket;

fn main() {
    let _ = sdl2::init(sdl::INIT_EVENTS);

    let port: u16 = 1234;

    // Init a server with a max of 100 connections
    if !rustnet::init_server(port, 100) {
        println!("Unable to start server on port {}", port);
        return
    } else {
        println!("Started server on port {}", port);
    }

    let mut connections: Vec<Socket> = Vec::new();

    let (tx, rx) = mpsc::channel();

    thread::spawn( move || {
        loop {
            let input = std::old_io::stdin().read_line().ok().expect("Failed to read input.");
            let _ = tx.send(input);
        }
    } );

    'main:loop {
        // Wait for a max of one millisecond for new messages to appear before moving on
        // It is VERY important to have some delay in the main method,
        // or it will run the processor at 100%
        // This is the best place for the delay, as there will be no delay
        // if there are new messages
        if rustnet::check_sockets(1) {
            let temp_client = rustnet::check_for_new_client();
            match temp_client {
                None => (),
                Some(socket) => {
                    println!("New connection.");

                    // Send the new connection some test stuff.
                    rustnet::clear_buffer();
                    rustnet::write_byte(1);
                    rustnet::write_float(43f32);
                    rustnet::write_uint(290u32);
                    rustnet::write_int(-312i32);
                    rustnet::send_message(&socket);

                    // It is recommended to wrap the socket in an outer class before pushing it.
                    connections.push(socket);
                }
            }

            // These are the sizes of our various incoming messages in bytes
            // not including the message id byte
            let msg_sizes = |msg_id: u8| {
                match msg_id {
                    1 => 12,
                    _ => 1, // _ must be > 0, or else a dropped packet would never be cleansed
                }
            };

            // Removes all disconnected clients
            let mut to_remove = Vec::new();
            for s in 0..connections.len() {
                // read_socket() reads all incomming data into the client's readbuffer
                // if false, then the client disconnected
                if !connections[s as usize].read_socket() {
                    println!("Lost connection to socket.");
                    to_remove.push(s);
                }
            }
            let mut offset: i32 = 0;
            for t in to_remove {
                connections.remove((t as i32 - offset) as usize);
                offset -= 1;
            }

            for i in 0..connections.len() {
                // Checks to see if the client has enough data on the buffer to read a message
                while connections[i].has_msg(&msg_sizes) {
                    // Read the message id
                    match connections[i].read_byte() {
                        1 => {
                            println!("Incoming float: {}", connections[i].read_float());
                            println!("Incoming uint: {}", connections[i].read_uint());
                            println!("Incoming int: {}", connections[i].read_int());
                        }
                        _ => println!("Unknown message."),
                    }
                }
            }
        }

        let recv = rx.try_recv();
        if recv.is_ok() {
            let out = recv.unwrap();
            let text = out.trim();
            if text == "quit" {
                break 'main;
            }
        }
    }
    println!("Server closed.");
}
