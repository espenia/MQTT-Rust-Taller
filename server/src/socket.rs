//! Estructura que almacena la información de cada client
use crate::json_helper::{read_topic_subs, write_topic_subs};
use crate::packets;
use serializer::mqtt_response::Mqtt5ReturnCodes::MqttRcProtocolError;
use serializer::mqtt_response::MqttError;
use serializer::{new_mqtt_header, Connect, Mqtt5ReturnCodes, MqttHeader, PacketType, Publish};
use std::error::Error;
use std::io::ErrorKind::WouldBlock;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::Sender;
use std::{cmp, thread};
use tracing::{error, info, warn};

/// Estructura que guarda la información de cada conexion. Un socket es un cliente conectado.
/// Posee dos TcpStream, uno para read y otro para write. Sender es el sender de un MPSC channel
/// donde se envían los Publish packets al servidor para que este reparta a los correspondientes clientes.
pub struct Socket {
    user: (u32, String),
    read: TcpStream,
    write: TcpStream,
    sender: Sender<Publish>,
    last_will: Vec<u8>,
}

impl Socket {
    pub fn new(connection: TcpStream, sender: Sender<Publish>, i: u32) -> Self {
        let read = connection;
        let write = read.try_clone().unwrap();
        Socket {
            read,
            write,
            user: (i, "".to_string()),
            sender,
            last_will: vec![],
        }
    }

    pub fn get_user(&self) -> u32 {
        self.user.0
    }

    pub fn get_write_stream(&self) -> TcpStream {
        self.write.try_clone().unwrap()
    }
    pub fn get_read_stream(&self) -> TcpStream {
        self.read.try_clone().unwrap()
    }

    pub fn handle_client(mut self) {
        let mut read = self.read.try_clone().unwrap();
        let mut write = self.write.try_clone().unwrap();
        let sender = self.sender.clone();
        thread::spawn(move || {
            let mut ret;
            loop {
                match self.read(&mut read, &mut write, &sender) {
                    Ok(b) => ret = b,
                    Err(_) => {
                        if self.write.shutdown(Shutdown::Write).is_ok() {}
                        if self.read.shutdown(Shutdown::Read).is_ok() {}
                        self.resolve_last_will(self.last_will.clone());
                        break;
                    }
                }
                if !ret {
                    break;
                }
            }
        });
    }

    fn read(
        &mut self,
        read: &mut TcpStream,
        write: &mut TcpStream,
        sender: &Sender<Publish>,
    ) -> Result<bool, Box<dyn Error>> {
        let data = Self::read_all(read)?;
        info!("Paquete recibido: {:?}", data.get_control_packet_type());
        return match self.read_array(data, write, sender, read) {
            Ok(b) => Ok(b),
            Err(_) => Ok(false),
        };
    }

    /// Leo una cantidad n de bytes del incoming stream, la cantidad n esta definida por el primer incoming byte.
    /// A partir de esos n bytes puedo decodificar el tipo de packet recibido.
    pub fn read_all(stream: &mut TcpStream) -> Result<MqttHeader, Box<dyn Error>> {
        let mut size_buf = [0_u8; 2];
        let msg_size: u32;
        let header: MqttHeader;
        let mut result: Vec<u8> = Vec::new();
        match stream.read_exact(&mut size_buf) {
            Ok(_) => {
                if size_buf[0] == 0 && size_buf[1] == 0 {
                    return Err(Box::new(MqttError {
                        error: Mqtt5ReturnCodes::MqttRcUnspecified,
                    }));
                }
                match new_mqtt_header(size_buf.to_vec()) {
                    Ok(h) => {
                        header = h;
                        msg_size = header.get_remaining_length() as u32;
                        result.append(&mut size_buf.to_vec())
                    }
                    Err(e) => {
                        return Err(Box::new(MqttError { error: e }));
                    }
                }
            }
            Err(ref e) => {
                if e.kind() == WouldBlock {
                    println!("Timeout");
                }
                return Err(Box::new(MqttError {
                    error: MqttRcProtocolError,
                }));
            }
        }

        // Leer del socket la cantidad de bytes que indica el header
        let mut bytes_read: u32 = 0;
        while bytes_read < msg_size {
            let max_limit = cmp::min(msg_size - bytes_read, 1024);
            let mut buf = vec![0; max_limit as usize].into_boxed_slice();
            match stream.read(&mut buf) {
                Ok(size) => {
                    let mut received = Vec::from(&buf[0..size]);
                    result.append(&mut received);
                    bytes_read += size as u32
                }
                Err(_) => {
                    return Err(Box::new(MqttError {
                        error: MqttRcProtocolError,
                    }))
                }
            }
        }

        match new_mqtt_header(result) {
            Ok(h) => Ok(h),
            Err(_) => Err(Box::new(MqttError {
                error: MqttRcProtocolError,
            })),
        }
    }
    /// Clasifico el tipo de packet recibido de read_all
    pub fn read_array(
        &mut self,
        header: MqttHeader,
        stream: &mut TcpStream,
        sender: &Sender<Publish>,
        read: &mut TcpStream,
    ) -> Result<bool, Box<dyn Error>> {
        //leo
        let user = self.user.clone();
        if (*user.1).to_string() == ""
            && header.get_control_packet_type() as u8 != PacketType::CONNECT as u8
        {
            warn!("[Server:Socket] No autorizado");
            return Err(Box::new(MqttError {
                error: Mqtt5ReturnCodes::MqttRcNotAuthorized,
            }));
        }
        match header.get_control_packet_type() {
            PacketType::CONNECT => {
                let connect = serializer::new_connect_by_hex(header)?;
                let ret = packets::connect::resolve_connect(connect.clone(), stream, user, read);
                match ret {
                    Ok(ret) => {
                        self.user = ret.clone();
                        if ret.1 != *"" {
                            self.handle_last_will(connect);
                        }
                    }
                    Err(_e) => {
                        error!("CONNECT resolve error");
                    }
                }
            }
            PacketType::PUBLISH => {
                let publish = serializer::new_publish_by_hex(header)?;
                let ret = packets::publish::resolve_publish(publish, stream, sender)?;
                if ret {
                    info!("PUBACK enviado correctamente.");
                }
            }
            PacketType::SUBSCRIBE => {
                let subscribe = serializer::new_subscribe_by_hex(header)?;
                let new_subs =
                    packets::subscribe::resolve_subscribe(stream, subscribe, user.clone())?;
                for topic in new_subs {
                    match packets::publish::send_retain_messages_to_sub(
                        topic,
                        stream,
                        (user.1).to_string(),
                    ) {
                        Ok(_) => {}
                        Err(_) => {
                            error!("error al mandar retain mensajes a subs")
                        }
                    }
                }
            }
            PacketType::UNSUSCRIBE => {
                let unsubscribe = serializer::new_unsubscribe_by_hex(header)?;
                packets::unsubscribe::resolve_unsubscribe(stream, unsubscribe, user)?;
            }
            PacketType::PINGREQ => {
                let pingresp = serializer::new_pingresp_by_hex();
                stream.write_all(&pingresp.get_data())?;
            }
            PacketType::DISCONNECT => {
                return Ok(false);
            }
            _ => {
                warn!("[Server:Socket] No deberia nunca llegar un paquete de este tipo, ignorado maquinola");
                return Ok(true);
            }
        }
        Ok(true)
    }

    /// En caso de que una conexion nueva posea last will/topic, esta funcion se encarga de generar un
    /// Publish packet data para luego enviar en caso de ungraceful disconnect.
    fn handle_last_will(&mut self, connect: Connect) {
        let flags = connect.get_connect_flags();
        if flags.get_will_flag() {
            let payload = connect.get_payload();
            let topic = payload.get_will_topic();
            let msg = payload.get_will_message();
            let topic_filter = serializer::new_topic_filter(topic.to_string())
                .ok()
                .unwrap();
            let mut subs;
            match read_topic_subs() {
                Ok(h) => {
                    subs = h;
                }
                Err(_) => {
                    error!("error al leer topic subs");
                    return;
                }
            }
            subs.entry(topic_filter.get_topic())
                .or_insert_with(|| Vec::new());
            match write_topic_subs(subs) {
                Ok(_) => {}
                Err(_) => {
                    error!("error al escribir topic subs")
                }
            }
            let payload_flags = serializer::new_publish_packet_flags(
                Some(flags.get_will_retain()),
                Some(flags.get_will_qos1()),
                Some(flags.get_will_qos2()),
                Some(false),
            )
            .ok()
            .unwrap();
            let publish = serializer::new_publish(payload_flags, topic_filter, msg.to_string());
            match publish {
                Ok(p) => self.last_will = p.get_data(),
                Err(_e) => error!("Error sending publish will"),
            }
        }
    }

    /// En caso de ungraceful disconnect acá se procesa el last will, se genera el correspondiente publish y se
    /// envía al server para ser procesado y repartido
    fn resolve_last_will(&self, data: Vec<u8>) {
        if !data.is_empty() {
            let header = serializer::new_mqtt_header(data);
            match header {
                Ok(h) => {
                    let publish = serializer::new_publish_by_hex(h);
                    match publish {
                        Ok(p) => {
                            if p.get_flags().get_retain() {
                                let payload_flags = p.get_flags();
                                let topic_filter = p.get_topic();
                                packets::publish::write_retain(
                                    payload_flags,
                                    topic_filter,
                                    p.clone(),
                                );
                            }
                            let _result = self.sender.send(p);
                        }
                        Err(_e) => {
                            error!("Error creating last_will packet");
                        }
                    }
                }
                Err(_e) => {
                    error!("Error creating last_will header");
                }
            }
        }
    }
}

impl Clone for Socket {
    fn clone(&self) -> Socket {
        Socket {
            read: self.read.try_clone().unwrap(),
            write: self.write.try_clone().unwrap(),
            user: self.user.clone(),
            sender: self.sender.clone(),
            last_will: self.last_will.clone(),
        }
    }
}
