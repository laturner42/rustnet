#![feature(core)]

extern crate sdl2_net;

//static MAX_BUFFER_SIZE: u16 = 512;

static mut write_buffer: [u8; 512] = [0; 512];
static mut buffer_index: u32 = 0;
static mut read_buffer: [u8; 512] = [0; 512];
static mut read_buffer_size: u32 = 0;

//static mut c_buffer: [u8; 512] = [0; 512]; //Vec<u8> = Vec::with_capacity(MAX_BUFFER_SIZE as usize);

static mut socket_set: sdl2_net::SocketSet = sdl2_net::SocketSet { opaque_ptr: &sdl2_net::_SDLNet_SocketSet };
static mut server_socket: TCPsocket = TCPsocket { opaque_ptr: &sdl2_net::_TCPsocket };

static mut is_server: bool = false;

pub use sdl2_net::TCPsocket;

pub fn read_socket<F: Fn(u8, u32) -> bool, J: Fn(u8, &TCPsocket) -> u32>(socket: &TCPsocket, c: F, f: J) -> bool {
    read_option_socket(socket, c, f)
}

pub fn read_server_socket<F: Fn(u8, u32) -> bool, J: Fn(u8, &TCPsocket) -> u32>(c: F, f: J) -> bool {
    unsafe {
        read_option_socket(&server_socket, c, f)
    }
}

fn read_option_socket<Able: Fn(u8, u32) -> bool, Doit: Fn(u8, &TCPsocket) -> u32>(socket: &TCPsocket, can_handle: Able, func: Doit) -> bool {

    if sdl2_net::socket_ready(socket) {
        unsafe {
            let rec_data = sdl2_net::tcp_recv(socket, &mut read_buffer[(read_buffer_size as usize)..512]);
            if rec_data > 0 {
                /*
                for i in 0..rec_data {
                    read_buffer[(read_buffer_size as i32 + i) as usize] = c_buffer[i as usize]
                }
                */
                read_buffer_size += rec_data as u32;
                while read_buffer_size > 0 {
                    if can_handle(peek_byte(), read_buffer_size){
                        func(read_byte(), socket);
                    } else { break; }
                }
                return true
            } else {
                remove_socket(&socket);
                sdl2_net::tcp_close(&socket);
                return false
            }
        }
    }

    true
}

pub fn peek_byte() -> u8 {
    unsafe{
        read_buffer[0]
    }
}

pub fn read_byte() -> u8 {
    let mut b: u8;
    unsafe {
        b = read_buffer[0];
        shift_buffer(1);
        read_buffer_size -= 1;
    }
    b
}

pub fn write_byte(b: u8) {
    unsafe {
        write_buffer[buffer_index as usize] = b;
        buffer_index += 1;
    }
}

pub fn read_float() -> f32 {
    let mut f: f32;
    unsafe{
        let mut bytes: [u8; 4] = [0; 4];//read_buffer[0..4];
        for i in range(0,bytes.len()) {
            bytes[i as usize] = read_byte();
        }
        f = std::mem::transmute(bytes);
    }
    f
}

pub fn write_float(f: f32) {
    unsafe {
        let bytes: [u8; 4] = std::mem::transmute(f);
        for byte in &bytes {
            write_buffer[buffer_index as usize] = *byte;
            buffer_index += 1;
        }
    }
}

pub fn clear_buffer() {
    unsafe {
        buffer_index = 0;
    }
}

pub fn send_ts_message() -> bool {
    unsafe {
        if is_server { return false }
        send_message_save(&server_socket, true)
    }
}

pub fn send_message(socket: &sdl2_net::TCPsocket) -> bool {
    send_message_save(socket, true)
}

pub fn send_message_save(socket: &sdl2_net::TCPsocket, clear: bool) -> bool{    
    let output;
    unsafe {
        let sent = sdl2_net::tcp_send(socket, &mut write_buffer[0..(buffer_index as usize)]) as u32;
        output = if sent < buffer_index { false } else { true };
        if clear { clear_buffer(); }
    }
    output
}


fn shift_buffer(shift: u32) {
    unsafe {
        for i in 0..read_buffer_size {
            read_buffer[i as usize] = read_buffer[(i+shift) as usize];
        }
    }
}

pub fn free_sockets() {
    unsafe {
        sdl2_net::free_socket_set(&socket_set);
        sdl2_net::tcp_close(&server_socket);
    }
}

pub fn check_for_new_client() -> Option<sdl2_net::TCPsocket> {
    unsafe {
        if !is_server { return None }
        if sdl2_net::socket_ready(&server_socket) {
            let pos_new_socket = sdl2_net::tcp_accept(&server_socket);

            let new_socket: sdl2_net::TCPsocket;

            match pos_new_socket {
                Some(s) => new_socket = s,
                None => return None,
            }

            sdl2_net::add_socket(&socket_set, &new_socket);

            return Some(new_socket)
        }
    }
    None
}

pub fn check_sockets() -> bool {
    unsafe {
        sdl2_net::check_sockets(&socket_set, 0) > 0
    }
}

fn remove_socket(socket: &sdl2_net::TCPsocket) {
    unsafe {
        sdl2_net::del_socket(&socket_set, &socket);
    }
}

pub fn init_server(port: u16, num_clients: u32) -> bool {
    let possible_ss = initialize(num_clients as i32);
    
    let s_set: sdl2_net::SocketSet;

    match possible_ss {
        None => return false,
        Some(ss) => s_set = ss,
    }
    unsafe {
        socket_set = s_set;
    }

    let possible_ip = sdl2_net::become_host(port);

    let mut ip: sdl2_net::IPaddress;

    match possible_ip {
        Some(i) => ip = i,
        None => return false,
    }

    let possible_socket = sdl2_net::tcp_open(&mut ip);

    let mut socket: sdl2_net::TCPsocket;

    match possible_socket {
        Some(s) => socket = s,
        None => return false,
    }

    unsafe {
        server_socket = socket;

        sdl2_net::add_socket(&socket_set, &server_socket);

        is_server = true;
    }

    true
}

pub fn init_client(host: &str, port: u16) -> bool {
    let possible_ss = initialize(1);

    let s_set: sdl2_net::SocketSet;

    match possible_ss{
        None => return false,
        Some(ss) => s_set = ss,
    }

    unsafe {
        socket_set = s_set;
    }

    let possible_ip = sdl2_net::resolve_host(host, port);

    let mut ip: sdl2_net::IPaddress;

    match possible_ip {
        Some(i) => ip = i,
        None => return false,
    }

    let possible_socket = sdl2_net::tcp_open(&mut ip);

    let mut socket: sdl2_net::TCPsocket;

    match possible_socket {
        Some(s) => socket = s,
        None => return false,
    }

    unsafe {
        server_socket = socket;

        sdl2_net::add_socket(&socket_set, &server_socket);
        
        is_server = false;
    }

    true
}

fn initialize(socket_set_size: i32) -> Option<sdl2_net::SocketSet> {
    if !sdl2_net::init() {
        println!("SDLNet init failure.");
        None
    } else {
        Some(sdl2_net::alloc_socket_set(socket_set_size))
    }
}
