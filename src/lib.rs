extern crate sdl2_net;

static MAX_BUFFER_SIZE: u16 = 512;

pub struct NetworkData{
    write_buffer: [u8; 512],
    buffer_index: u32,
    read_buffer: [u8; 512],
    read_buffer_size: u32,

    c_buffer: Vec<u8>,

    socket_set: sdl2_net::SocketSet,
    pub socket: sdl2_net::TCPsocket,

    is_server: bool,
}

pub fn read_socket<F: Fn(u8, u32) -> bool, J: Fn(&NetworkData) -> u32>(net:&mut NetworkData, socket: &sdl2_net::TCPsocket, c: F, f: J) -> bool {
    read_option_socket(net, Some(socket), c, f)
}

pub fn read_server_socket<F: Fn(u8, u32) -> bool, J: Fn(&NetworkData) -> u32>(net:&mut NetworkData, c: F, f: J) -> bool {
    read_option_socket(net, None, c, f)
}


fn read_option_socket<F: Fn(u8, u32) -> bool, J: Fn(&NetworkData) -> u32>(net: &mut NetworkData, osocket: Option<&sdl2_net::TCPsocket>, can_handle: F, func: J) -> bool {

    let socket: &sdl2_net::TCPsocket;

    match osocket {
        None => socket = &(net.socket),
        Some(s) => socket = s,
    }

    if sdl2_net::socket_ready(socket) {
        let rec_data = sdl2_net::tcp_recv(socket, net.c_buffer.as_mut_ptr(), MAX_BUFFER_SIZE as i32);
        if rec_data > 0 {
            for i in 0..rec_data {
                net.read_buffer[(net.read_buffer_size as i32 + i) as usize] = net.c_buffer[i as usize]
            }
            net.read_buffer_size += rec_data as u32;
            while net.read_buffer_size > 0 {
                if can_handle(peek_byte(net), net.read_buffer_size){
                    func(&net);
                } else { break; }
            }
            return true
        } else {
            remove_socket(&net, &(net.socket));
            sdl2_net::tcp_close(&(net.socket));
            return false
        }
    }

    true
}

pub fn peek_byte(net_data: &NetworkData) -> u8 {
    net_data.read_buffer[0]
}

pub fn read_byte(net_data: &mut NetworkData) -> u8 {
    let b = net_data.read_buffer[0];
    shift_buffer(net_data, 1);
    b
}

pub fn write_byte(net_data: &mut NetworkData, b: u8) {
    net_data.write_buffer[net_data.buffer_index as usize] = b;
    net_data.buffer_index += 1;
}

pub fn clear_buffer(net_data: &mut NetworkData) {
    net_data.buffer_index = 0;
}

pub fn send_message(net_data: &mut NetworkData) -> bool {
    send_message_save(net_data, None, true)
}

pub fn send_message_socket(net_data: &mut NetworkData, socket: &sdl2_net::TCPsocket) -> bool {
    send_message_save(net_data, Some(socket), true)
}

pub fn send_message_save(net_data: &mut NetworkData, osocket: Option<&sdl2_net::TCPsocket>, clear: bool) -> bool{
    let socket: &sdl2_net::TCPsocket;

    match osocket {
        None => socket = &(net_data.socket),
        Some(s) => socket = s,
    }
    
    let sent = sdl2_net::tcp_send(socket, net_data.write_buffer.as_mut_ptr(),
    net_data.buffer_index) as u32;
    let output: bool = if sent < net_data.buffer_index { false } else { true };
    if clear { net_data.buffer_index = 0; } // equivalent of clear_bufer(), but rust...
    output
}


fn shift_buffer(net_data: &mut NetworkData, shift: u32) {
    for i in 0..net_data.read_buffer_size {
        net_data.read_buffer[i as usize] = net_data.read_buffer[(i+shift) as usize];
    }
}

pub fn free_sockets(net_data: &mut NetworkData) {
    sdl2_net::free_socket_set(&(net_data.socket_set));
    sdl2_net::tcp_close(&(net_data.socket));
}

pub fn check_for_new_client(net_data: &NetworkData) -> Option<sdl2_net::TCPsocket> {
    if !net_data.is_server { return None }
    if sdl2_net::socket_ready(&(net_data.socket)) {
        let pos_new_socket = sdl2_net::tcp_accept(&(net_data.socket));

        let new_socket: sdl2_net::TCPsocket;

        match pos_new_socket {
            Some(s) => new_socket = s,
            None => return None,
        }

        sdl2_net::add_socket(&(net_data.socket_set), &(new_socket));

        return Some(new_socket)
    }
    None
}

pub fn check_sockets(net_data: &NetworkData) -> bool {
    sdl2_net::check_sockets(&(net_data.socket_set), 0) > 0
}

fn remove_socket(net_data: &NetworkData, socket: &sdl2_net::TCPsocket) {
    sdl2_net::del_socket(&(net_data.socket_set), &socket);
}

pub fn init_server(port: u16, num_clients: u32) -> Option<NetworkData> {
    let possible_ss = initialize(num_clients as i32);
    
    let socket_set: sdl2_net::SocketSet;

    match possible_ss{
        None => return None,
        Some(ss) => socket_set = ss,
    }

    let possible_ip = sdl2_net::become_host(port);

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

    sdl2_net::add_socket(&socket_set, &socket);

    Some(new_net_data(socket_set, socket, true))

}

pub fn init_client(host: &str, port: u16) -> Option<NetworkData> {
    let possible_ss = initialize(1);

    let socket_set: sdl2_net::SocketSet;

    match possible_ss{
        None => return None,
        Some(ss) => socket_set = ss,
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

    sdl2_net::add_socket(&socket_set, &socket);

    Some(new_net_data(socket_set, socket, false))
}

fn new_net_data(socket_set: sdl2_net::SocketSet, socket: sdl2_net::TCPsocket, is_server: bool) -> NetworkData {
    NetworkData{   write_buffer : [0; 512],    buffer_index : 0,
                        read_buffer : [0; 512],     read_buffer_size : 0,
                        c_buffer : Vec::with_capacity(MAX_BUFFER_SIZE as usize),
                        socket_set : socket_set,    socket : socket,
                        is_server: is_server}
}

fn initialize(socket_set_size: i32) -> Option<sdl2_net::SocketSet> {
    if !sdl2_net::init() {
        println!("SDLNet init failure.");
        None
    } else {
        Some(sdl2_net::alloc_socket_set(socket_set_size))
    }
}
