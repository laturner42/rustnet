#![feature(core)]

extern crate sdl2_net;

//static MAX_BUFFER_SIZE: u16 = 512;

static mut write_buffer: [u8; 512] = [0; 512];
static mut buffer_index: u32 = 0;

static mut socket_set: sdl2_net::SocketSet = sdl2_net::SocketSet { opaque_ptr: &sdl2_net::_SDLNet_SocketSet };
static mut server_socket: TCPsocket = TCPsocket { opaque_ptr: &sdl2_net::_TCPsocket };

static mut is_server: bool = false;

pub use sdl2_net::TCPsocket;

pub struct SocketWrapper{
    socket: TCPsocket,
    read_buffer: [u8; 512],
    read_buffer_size: u32,
}

impl SocketWrapper {
    pub fn socket(&self) -> &TCPsocket {
        &self.socket
    }

    pub fn tcp_socket(&self) -> &TCPsocket {
        &self.socket
    }

    pub fn read_buffer(&mut self) -> &mut [u8; 512] {
        &mut self.read_buffer
    }

    pub fn read_buffer_size(&self) -> u32 {
        self.read_buffer_size
    }

    fn read_data(&mut self) -> i32 {
        sdl2_net::tcp_recv(&self.socket, &mut self.read_buffer[(self.read_buffer_size as usize)..512])
    }

    pub fn read_socket(&mut self) -> bool{
        if sdl2_net::socket_ready(&self.socket) {
            let rec_data = self.read_data();
            if rec_data > 0 {
                return true
            } else {
                remove_socket(&self.socket);
                sdl2_net::tcp_close(&self.socket);
                return false
            }
        }

        true
    }

    pub fn has_msg<Able: Fn(u8) -> u32>(&self, msg_size: &Able) -> bool{
        msg_size(self.peek_byte()) < self.read_buffer_size
    }

    pub fn peek_byte(&self) -> u8 {
        self.read_buffer[0]
    }

    pub fn read_byte(&mut self) -> u8 {
        let mut b: u8;
        b = self.read_buffer[0];
        self.shift_buffer(1);
        self.read_buffer_size -= 1;
        b
    }

    pub fn read_float(&mut self) -> f32 {
        let mut f: f32;
        unsafe{
            let mut bytes: [u8; 4] = [0; 4];//read_buffer[0..4];
            for i in range(0, bytes.len()) {
                bytes[i as usize] = self.read_byte();
            }
            f = std::mem::transmute(bytes);
        }
        f
    }

    fn shift_buffer(&mut self, shift: u32) {
        for i in 0..self.read_buffer_size {
            self.read_buffer[i as usize] = self.read_buffer[(i+shift) as usize];
        }
    }

}

pub fn new_socket_wrapper(socket: TCPsocket) -> SocketWrapper{
    SocketWrapper{socket: socket, read_buffer: [0; 512], read_buffer_size: 0}
}


pub fn write_byte(b: u8) {
    unsafe {
        write_buffer[buffer_index as usize] = b;
        buffer_index += 1;
    }
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

/*
pub fn send_ts_message() -> bool {
    unsafe {
        if is_server { return false }
        send_message_save(&server_socket, true)
    }
}
*/

pub fn send_message(wrapper: &SocketWrapper) -> bool {
    send_message_save(wrapper, true)
}

pub fn send_message_save(wrapper: &SocketWrapper, clear: bool) -> bool{    
    let output;
    unsafe {
        let sent = sdl2_net::tcp_send(wrapper.socket(), &mut write_buffer[0..(buffer_index as usize)]) as u32;
        output = if sent < buffer_index { false } else { true };
        if clear { clear_buffer(); }
    }
    output
}

pub fn free_sockets() {
    unsafe {
        sdl2_net::free_socket_set(&socket_set);
        sdl2_net::tcp_close(&server_socket);
    }
}

pub fn check_for_new_client() -> Option<SocketWrapper> {
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

            return Some(new_socket_wrapper(new_socket))
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

pub fn init_client(host: &str, port: u16) -> Option<SocketWrapper> {
    let possible_ss = initialize(1);

    let s_set: sdl2_net::SocketSet;

    match possible_ss{
        None => return None,
        Some(ss) => s_set = ss,
    }

    unsafe {
        socket_set = s_set;
    }

    let possible_ip = sdl2_net::resolve_host(host, port);

    let mut ip: sdl2_net::IPaddress;

    match possible_ip {
        Some(i) => ip = i,
        None => return None,
    }

    let possible_socket = sdl2_net::tcp_open(&mut ip);

    let mut socket: sdl2_net::TCPsocket;

    match possible_socket {
        Some(s) => socket = s,
        None => return None,
    }

    unsafe {
        //server_socket = socket;

        sdl2_net::add_socket(&socket_set, &server_socket);
        
        is_server = false;
    }

    Some(new_socket_wrapper(socket))
}

fn initialize(socket_set_size: i32) -> Option<sdl2_net::SocketSet> {
    if !sdl2_net::init() {
        println!("SDLNet init failure.");
        None
    } else {
        Some(sdl2_net::alloc_socket_set(socket_set_size))
    }
}
