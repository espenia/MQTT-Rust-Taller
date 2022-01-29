//! Estructura del Server
use crate::json_helper::{read_q_messages, read_topic_subs, read_users, write_q_messages};
use crate::packets::user_qos::UserQos;
use crate::socket::Socket;
use serializer::Publish;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{error, info};

/// Estructura del servidor, contiene el socket donde escucha incoming connections,
/// un vector con cada cliente conectado y el sender del MPSC channel que se le envía
#[derive(Clone)]
pub(crate) struct Server {
    address: String,
    port: u16,
    socket: Arc<TcpListener>,
    connections: Arc<Mutex<Vec<Socket>>>,
    sender: Sender<Publish>,
}

impl Server {
    ///Al iniciar el server, crea un vector de sockets, y empieza a escuchar en el puerto indicado en el archivo de config
    /// se establece un channel para comunicarse entre el socket y el server, y se crea un thread para hacer el handling de publish.
    pub fn new() -> Self {
        info!("[Server]Inicializando server.");
        let config = decode_config();
        let binding = Self::server_connect(
            config[0][1].to_string() + ":" + config[1][1].to_string().as_str(),
        );
        if binding.is_err() {
            error!("Error al realizar conexion.");
        }
        let (sender, receiver) = mpsc::channel::<Publish>();
        let connections: Vec<Socket> = Vec::new();

        let server = Server {
            address: config[0][1].to_string(),
            port: config[1][1].parse().unwrap(),
            socket: Arc::new(binding.unwrap()),
            connections: Arc::new(Mutex::new(connections)),
            sender,
        };

        let connection_ref = Arc::clone(&server.connections);
        thread::spawn(move || {
            Self::receive_packets(connection_ref, receiver);
        });
        server
    }

    fn server_connect(address: String) -> std::io::Result<TcpListener> {
        TcpListener::bind(address)
    }

    ///Se escucha por nuevos clientes, se crea el socket, y se envia al handling del cliente.
    pub fn listen(&mut self) {
        let mut users: HashMap<u32, String> = HashMap::new();
        if let Ok(h) = read_users() {
            users = h;
        }
        let mut i = *users.keys().max().unwrap() + 1;
        loop {
            for stream in self.socket.incoming() {
                let new_client = Socket::new(stream.unwrap(), self.sender.clone(), i);
                let cloned_client = new_client.clone();
                info!("Nueva conexion!");
                match self.connections.lock() {
                    Ok(mut conn_vec) => {
                        conn_vec.push(cloned_client);
                    }
                    Err(_) => {
                        error!("Error en listen.")
                    }
                }
                new_client.handle_client();
                users.insert(i, "".to_string());
                i += 1;
            }
        }
    }

    /// Lee el receiver del MPSC channel esperando incoming publish packets y los procesa.
    fn receive_packets(connections: Arc<Mutex<Vec<Socket>>>, receiver: Receiver<Publish>) {
        loop {
            match receiver.recv() {
                Ok(mut packet) => {
                    let topic = packet.clone().get_topic().get_topic();
                    let subs;
                    match read_topic_subs() {
                        Ok(h) => {
                            subs = h;
                        }
                        Err(_) => continue,
                    }
                    let userhash;
                    match read_users() {
                        Ok(h) => {
                            userhash = h;
                        }
                        Err(_) => continue,
                    }
                    save_messages(
                        subs.clone(),
                        userhash.clone(),
                        connections.clone(),
                        packet.clone(),
                    );
                    let users = subs[&topic].clone();
                    for user in users {
                        let sockets = connections.lock().unwrap();
                        for socket in sockets.to_vec() {
                            if !userhash.contains_key(&socket.get_user()) {
                                continue;
                            }
                            let u = userhash[&socket.get_user()].clone();
                            if u == user.get_user() {
                                if packet.get_flags().get_qos() == 0 {
                                    match socket.get_write_stream().write(&packet.get_data()) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            error!("error al enviar a subs")
                                        }
                                    }
                                } else if user.get_qos() == 1 {
                                    let ret = socket.get_write_stream().write(&packet.get_data());
                                    if ret.is_err() {
                                        error!("error al enviar a subs")
                                    } else {
                                        match Socket::read_all(&mut socket.get_read_stream()) {
                                            Ok(h) => {
                                                if let serializer::PacketType::PUBACK =
                                                    h.get_control_packet_type()
                                                {
                                                }
                                            }
                                            Err(_) => {
                                                error!("error al leer")
                                            }
                                        }
                                    }
                                } else {
                                    packet = packet.set_qos_flag(0);
                                    match socket.get_write_stream().write(&packet.get_data()) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            error!("error al enviar a subs")
                                        }
                                    }
                                }
                            }
                        }
                    }
                    info!("[Server] Received packet publish");
                }
                Err(_) => return,
            }
        }
    }
}

fn save_messages(
    subs: HashMap<String, Vec<UserQos>>,
    mut users: HashMap<u32, String>,
    connections: Arc<Mutex<Vec<Socket>>>,
    packet: Publish,
) {
    if let Ok(mut h) = read_q_messages() {
        for socket in connections.lock().unwrap().to_vec() {
            users.remove(&socket.get_user());
        }
        let topic = packet.get_topic().get_topic();
        let userqos = subs[&topic].clone();
        for user in userqos {
            let mut contains = false;
            for u in users.values() {
                if u.clone() == user.get_user() {
                    contains = true;
                }
            }
            if contains {
                let mut mess = h[&user.get_user().clone()].clone();
                mess.push(packet.get_data());
                h.remove(&user.get_user().clone());
                h.insert(user.get_user().clone(), mess);
            }
        }
        if write_q_messages(h).is_ok() {}
    }
}

fn decode_config() -> Vec<Vec<String>> {
    let file = File::open("config.txt").expect("Path no existe!");
    let mut buffer = String::new();
    let mut reader = BufReader::new(file);
    let result = reader.read_to_string(&mut buffer);
    if result.is_err() {
        error!(
            "[Server] Obteniendo configuraciones: Error {:?}",
            result.err().unwrap()
        );
    }
    let data: Vec<String> = buffer.split(',').map(|s| s.to_string()).collect();
    let mut server: Vec<String> = data[0].split(':').map(|s| s.to_string()).collect();
    let mut port: Vec<String> = data[1].split(':').map(|s| s.to_string()).collect();
    for i in server.iter_mut() {
        *i = i
            .trim()
            .replace("^\\s+|\\s+$|\\s*(\n)\\s*|(\\s)\\s*", "")
            .replace("\t", "");
    }
    for i in port.iter_mut() {
        *i = i
            .trim()
            .replace("^\\s+|\\s+$|\\s*(\n)\\s*|(\\s)\\s*", "")
            .replace("\t", "");
    }
    if server[0] != "server" || port[0] != "port" {
        error!(
            Error = "[Server] Estructura no valida.",
            "Obteniendo configuraciones:"
        );
        panic!("error estructura no valida");
    }
    info!(
        "[Server] Obteniendo configuraciones: Servidor:{:?}; Puerto:{:?}",
        server[1], port[1]
    );
    return vec![server, port];
}
