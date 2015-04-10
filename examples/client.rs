extern crate sdl2;
extern crate rustnet;

use rustnet::SocketWrapper as Socket;

fn main() {
    let _ = sdl2::init(sdl2::INIT_EVENTS);
    
    let port: u16 = 1234;
    let ip = "127.0.0.1";

    let option = rustnet::init_client(ip, port);
    let mut socket: Socket;

    match option {
        Some(sock) => socket = sock,
        None => {
            println!("Unable to connect to {}:{}", ip, port);
            return;
        },
    }

    let msg_size = |msg_id: u8| -> u32 {
        match msg_id {
            1 => 12,
            _ => 1,
        }
    };

    'main:loop {
        if rustnet::check_sockets(1) {
            if !socket.read_socket() {
                println!("Connection lost.");
                break 'main;
            } else {
                while socket.has_msg(&msg_size) {
                    match socket.read_byte() {
                        1 => {
                            println!("Received float {}", socket.read_float());
                            println!("Received uint {}", socket.read_uint());
                            println!("Received int {}", socket.read_int());
                        },
                        2 => {
                        },
                        _ => println!("Unknown message"),
                    }
                }
            }
        }
    }
}
