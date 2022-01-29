use serializer::mqtt_response::Mqtt5ReturnCodes::MqttRcProtocolError;
use serializer::mqtt_response::MqttError;
use serializer::{new_mqtt_header, new_puback, MqttHeader, PacketType, SubackReturnCode};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{cmp, fs, thread};
use tracing::{error, info, warn};
extern crate gtk;
use glib::Sender;

pub(crate) struct Client {
    pub(crate) socket: TcpStream,
}

impl Client {
    pub fn new() -> Self {
        let config = decode_config().unwrap();
        let connection =
            //Self::client_run(address + ":" + port.as_str());
            Self::client_run(config[0][1].to_string() + ":" + config[1][1].to_string().as_str());
        if connection.is_err() {
            error!("Error al realizar conexion.");
        } else {
            info!("Conectado al servidor.");
        }
        Client {
            socket: connection.unwrap(),
        }
    }
    fn client_run(address: String) -> std::io::Result<TcpStream> {
        TcpStream::connect(address)
    }

    ///Lee mensajes, identifica el tipo y llama al resolve para realizar la logica en packets.rs
    /// Utiliza un channel para enviar mensajes, errores y otras cosas a la interfaz, de la aplicacion.
    pub fn await_packets(
        stream: &mut TcpStream,
        tx: &Sender<String>,
        write: &Arc<Mutex<TcpStream>>,
    ) {
        let resp = Self::read_all(stream);
        match resp {
            Ok(header) => match header.get_control_packet_type() {
                PacketType::CONNACK => {
                    let connack = serializer::new_connack_by_hex(header);
                    match connack {
                        Ok(connack) => {
                            info!(
                                "Respuesta recibida: Paquete {:?} \n\
                                       Connect Aknowledge Flags: {:?} \n\
                                       Return codes: {:?}",
                                connack.get_packet_type(),
                                connack.get_connect_acknowledge_flags(),
                                connack.get_connect_return_codes().is_accepted()
                            );

                            if connack.get_connect_return_codes().is_accepted() {
                                tx.send("CONNACK|Conexion aceptada!".to_string())
                                    .expect("Couldn't send data to channel");
                            } else {
                                let reason = match connack.get_connect_return_codes().get_reason() {
                                    0x00 => "ConnectionAccepted".to_string(),
                                    0x01 => "InvalidProtocol".to_string(),
                                    0x02 => "IdentifierRejected".to_string(),
                                    0x03 => "ServerUnavailable".to_string(),
                                    0x04 => "BadUserNameOrPassword".to_string(),
                                    0x05 => "NotAuthorized".to_string(),
                                    _ => "CloseConnection".to_string(),
                                };
                                tx.send(
                                    "CONNACK|Conexion rechazada \n Razon: ".to_owned() + &reason,
                                )
                                .expect("Couldn't send data to channel");
                                // tx.send(String::from(
                                //     "CONNACK|Conexion rechazada \n Razon: ".to_owned()
                                //         + &connack
                                //             .get_connect_return_codes()
                                //             .get_reason()
                                //             .to_string(),
                                // ))
                                // .expect("Couldn't send data to channel");
                            }
                        }
                        Err(e) => {
                            error!("Error en crear CONNACK: {:?}", e);
                            tx.send("CONNACK|Error en el paquete CONNACK recibido.".to_string())
                                .expect("Couldn't send data to channel");
                        }
                    }
                }
                PacketType::PUBACK => {
                    // Si me llega un PUBACK es porque se publico exitosamente. No me importa lo que dice adentro.
                    info!("Respuesta recibida: Paquete PUBACK.");
                    tx.send("PUBACK|Published Succesfully".to_string())
                        .expect("Couldn't send data to channel");
                }
                PacketType::SUBACK => {
                    let suback = serializer::new_suback_by_hex(header);
                    match suback {
                        Ok(suback) => {
                            let mut return_codes_str = "".to_string();
                            let return_codes = suback.get_return_codes();
                            let mut contains_failures = false;
                            for code in return_codes {
                                match code {
                                    SubackReturnCode::Failure => {
                                        return_codes_str += "Failure, ";
                                        contains_failures = true;
                                    }
                                    SubackReturnCode::MaxQoS1 => return_codes_str += "MaxQos1, ",
                                    SubackReturnCode::MaxQoS0 => return_codes_str += "MaxQos0, ",
                                    _ => {
                                        error!("Suback return code invalido me llego un qos2")
                                    }
                                }
                            }
                            if contains_failures {
                                error!(
                                    "Error al subscribrse en topics codes {:?}",
                                    return_codes_str
                                );
                                tx.send(
                                    "SUBACK|Received Failures, following return codes ".to_string()
                                        + return_codes_str.as_str(),
                                )
                                .expect("Couldn't send data to channel")
                            } else {
                                info!("Recibi subacks con return codes {:?}", return_codes_str);
                                tx.send(
                                    "SUBACK|Subscribed Succesfully with return codes: ".to_string()
                                        + return_codes_str.as_str(),
                                )
                                .expect("Couldn't send data to channel")
                            }
                        }
                        Err(e) => {
                            error!("Error en recibir SUBACK: {:?}", e);
                            tx.send("SUBACK|Error en el paquete SUBACK recibido.".to_string())
                                .expect("Couldn't send data to channel");
                        }
                    }
                }
                PacketType::UNSUBACK => {
                    info!("Me llego un unsuback");
                    tx.send("UNSUBACK|Unubscribed Succesfully".to_string())
                        .expect("Couldn't send data to channel");
                }
                PacketType::PUBLISH => {
                    let publish = serializer::new_publish_by_hex(header);
                    match publish {
                        Ok(publish) => {
                            tx.send(format!(
                                "PUBLISH|Topic: {:?} :: Message: {:?}",
                                publish.get_topic().get_topic(),
                                publish.get_payload()
                            ))
                            .expect("Couldn't send data to channel");
                            if publish.get_flags().get_qos() == 1 {
                                let puback = new_puback();
                                if write.lock().unwrap().write(&puback.get_data()).is_ok() {}
                            }
                        }
                        Err(e) => {
                            error!("Error en recibir PUBLISH: {:?}", e);
                            tx.send("PUBLISH|Error en el paquete PUBLISH recibido.".to_string())
                                .expect("Couldn't send data to channel");
                        }
                    }
                }
                PacketType::PINGRESP => {
                    info!("Me llego un PINGRESP.");
                }
                _ => {
                    //No me deberian llegar otro tipo de mensajes.
                    warn!("Llego un tipo de mensaje inesperado.");
                }
            },
            Err(e) => {
                error!("Error en respuesta: {:?}", e);
                thread::sleep(Duration::from_millis(500));
                // No envio nada.
            }
        }
    }

    fn read_all(stream: &mut TcpStream) -> Result<MqttHeader, Box<dyn Error>> {
        let mut size_buf = [0_u8; 2];
        let msg_size: u32;
        let header: MqttHeader;
        let mut result: Vec<u8> = Vec::new();
        match stream.read_exact(&mut size_buf) {
            Ok(_) => match new_mqtt_header(size_buf.to_vec()) {
                Ok(h) => {
                    header = h;
                    msg_size = header.get_remaining_length() as u32;
                    result.append(&mut size_buf.to_vec())
                }
                Err(e) => {
                    return Err(Box::new(MqttError { error: e }));
                }
            },
            Err(_) => {
                return Err(Box::new(MqttError {
                    error: MqttRcProtocolError,
                }))
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
}

pub fn decode_config() -> Result<Vec<Vec<String>>, bool> {
    let file: File;
    match File::open("config.txt") {
        Ok(f) => {
            file = f;
        }
        Err(_) => {
            return Err(false);
        }
    }
    let mut buffer = String::new();
    let mut reader = BufReader::new(file);
    match reader.read_to_string(&mut buffer) {
        Ok(_) => {}
        Err(_) => {
            return Err(false);
        }
    }

    let data: Vec<String> = buffer.split(',').map(|s| s.to_string()).collect();
    let mut server: Vec<String> = data[0].split(':').map(|s| s.to_string()).collect();
    let mut port: Vec<String> = data[1].split(':').map(|s| s.to_string()).collect();
    let mut user: Vec<String> = data[2].split(':').map(|s| s.to_string()).collect();
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
    for i in user.iter_mut() {
        *i = i
            .trim()
            .replace("^\\s+|\\s+$|\\s*(\n)\\s*|(\\s)\\s*", "")
            .replace("\t", "");
    }
    if server[0] != *"server" || port[0] != *"port" || user[0] != *"user" {
        panic!("error estructura no valida");
    }
    Ok(vec![server, port, user])
}

pub fn update_changes(server: String, port: String) -> bool {
    let current_content;
    match decode_config() {
        Ok(c) => {
            current_content = c;
        }
        Err(_) => {
            return false;
        }
    }
    let content = "server:".to_owned()
        + &server
        + ","
        + "port:"
        + &port
        + ","
        + "user:"
        + &current_content[2][1];
    return match fs::write("config.txt", content.as_bytes()) {
        Ok(_) => {
            info!("config actualizada {:?}", content);
            true
        }
        Err(_) => {
            error!("error al actualizar config {:?}", content);
            false
        }
    };
}

pub fn update_changes_user(user: String) -> bool {
    let current_content;
    match decode_config() {
        Ok(c) => {
            current_content = c;
        }
        Err(_) => {
            return false;
        }
    }
    let content = "server:".to_owned()
        + &current_content[0][1]
        + ","
        + "port:"
        + &current_content[1][1]
        + ","
        + "user:"
        + &user;
    return match fs::write("config.txt", content.as_bytes()) {
        Ok(_) => {
            info!("config actualizada {:?}", content);
            true
        }
        Err(_) => {
            error!("error al actualizar config {:?}", content);
            false
        }
    };
}
