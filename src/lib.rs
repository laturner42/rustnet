extern crate sdl2_net;

static MAX_BUFFER_SIZE: u16 = 512;

pub struct NetworkData{
    write_buffer: [u8; 512],
    buffer_index: u32,
    read_buffer: [u8; 512],
    read_buffer_size: u32,

    socket_set: sdl2_net::SocketSet,
    socket: sdl2_net::TCPsocket,

    is_server: bool,
}

pub fn rnet_check_for_new_client(net_data: NetworkData) -> Option<sdl2_net::TCPsocket> {
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

pub fn rnet_init_server(port: u16, num_clients: u32) -> Option<NetworkData> {
    let possible_ss = rnet_initialize(num_clients as i32);
    
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

    Some(NetworkData{   write_buffer : [0; 512],    buffer_index : 0,
                        read_buffer : [0; 512],     read_buffer_size : 0,
                        socket_set : socket_set,    socket : socket,
                        is_server : true})
}

pub fn rnet_init_client(host: &str, port: u16) -> Option<NetworkData> {
    let possible_ss = rnet_initialize(1);

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

    Some(NetworkData{   write_buffer : [0; 512],    buffer_index : 0,
                        read_buffer : [0; 512],     read_buffer_size : 0,
                        socket_set : socket_set,    socket : socket,
                        is_server: false})
}

pub fn rnet_readsocket(socket: sdl2_net::TCPsocket, net_data: NetworkData) -> bool {
    true
}

fn rnet_initialize(socket_set_size: i32) -> Option<sdl2_net::SocketSet> {
    if !sdl2_net::init() {
        println!("SDLNet init failure.");
        None
    } else {
        Some(sdl2_net::alloc_socket_set(socket_set_size))
    }
}
