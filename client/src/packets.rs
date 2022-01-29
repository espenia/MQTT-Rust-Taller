use crate::client;
use serializer::mqtt_response::MqttError;
use serializer::{
    new_connect, new_topic_filter, new_topic_filter_with_qos, ConnectFlag, Mqtt5ReturnCodes,
    PayloadConnect, PublishFlag, TopicFilter,
};
use std::error::Error;
use std::io::Write;
use std::net::TcpStream;
use tracing::{error, info};

pub fn send_publish(
    stream: &mut TcpStream,
    flags: PublishFlag,
    topic: TopicFilter,
    payload: String,
) -> Result<usize, Mqtt5ReturnCodes> {
    let publish = serializer::new_publish(flags, topic, payload)?;
    return match stream.write(&publish.get_data()) {
        Ok(size) => {
            publish.get_topic().get_topic();
            publish.get_payload();
            publish.get_flags().get_qos();
            info!(
                "Enviado Paquete PUBLISH:\n\
            Topic: {:?} \n\
            Payload: {:?} \n\
            QoS: {:?} \n\
            Retain: {:?}",
                publish.get_topic().get_topic(),
                publish.get_payload(),
                publish.get_flags().get_qos(),
                publish.get_flags().get_retain()
            );
            Ok(size)
        }
        Err(_) => {
            error!("Error al enviar PUBLISH");
            Err(Mqtt5ReturnCodes::MqttRcServerUnavailable)
        }
    };
}

pub fn send_subscribe(
    stream: &mut TcpStream,
    topics: Vec<String>,
    qos: u8,
) -> Result<usize, Box<dyn Error>> {
    let mut topic_vec = vec![];
    for t in topics {
        match new_topic_filter_with_qos(t, qos) {
            Ok(topic) => {
                topic_vec.push(topic.clone());
            }
            Err(e) => {
                error!("Topic invalido");
                return Err(Box::new(MqttError { error: e }));
            }
        }
    }
    let subscribe = serializer::new_subscribe(topic_vec)?;
    return match stream.write(&subscribe.get_data()) {
        Ok(s) => {
            let mut topics: Vec<String> = vec![];
            for i in subscribe.get_topics() {
                topics.push(i.get_topic());
            }
            info!(
                "Enviando Paquete Subscribe:\n\
            Topics: {:?}
            ",
                topics.join("\n\"")
            );
            Ok(s)
        }
        Err(e) => {
            error!("Error al enviar Subscribe");
            Err(e.into())
        }
    };
}

pub fn send_unsubscribe(
    stream: &mut TcpStream,
    topics: Vec<String>,
) -> Result<usize, Box<dyn Error>> {
    let mut topic_vec = vec![];
    for t in topics {
        match new_topic_filter(t) {
            Ok(topic) => {
                topic_vec.push(topic.clone());
            }
            Err(e) => {
                error!("Topic invalido");
                return Err(Box::new(MqttError { error: e }));
            }
        }
    }
    let unsubscribe = serializer::new_unsubscribe(topic_vec)?;
    return match stream.write(&unsubscribe.get_data()) {
        Ok(s) => {
            let mut topics: Vec<String> = vec![];
            for i in unsubscribe.get_topic_filters() {
                topics.push(i.get_topic());
            }
            info!(
                "Enviando Paquete Unsubscribe:\n\
            Topics: {:?}
            ",
                topics.join("\n\"")
            );

            Ok(s)
        }
        Err(e) => {
            error!("Error al enviar Unsubscribe");
            Err(e.into())
        }
    };
}

pub fn send_connect(
    stream: &mut TcpStream,
    connect_flag: ConnectFlag,
    connect_payload: PayloadConnect,
) -> Result<usize, Mqtt5ReturnCodes> {
    let connect_packet = new_connect(connect_flag, connect_payload);
    match connect_packet {
        Ok(connect_packet) => {
            let data = connect_packet.get_data();
            return match stream.write(&data) {
                Ok(s) => {
                    info!(
                        "Enviando Paquete CONNECT:\n\
                    Client ID: {:?} \n\
                    Username: {:?} \n\
                    Password: {:?} \n\
                    Will Topic: {:?} \n\
                    Will Message: {:?} \n\
                    Clean Session: {:?} \n\
                    Will Retain: {:?} \n\
                    Will Flag: {:?} \n\
                    Will QoS b1: {:?} \n\
                    Will QoS b2: {:?} \n\
                    Username Flag: {:?} \n\
                    Password Flag: {:?}",
                        connect_packet.get_payload().get_client_identifier(),
                        connect_packet.get_payload().get_username(),
                        connect_packet.get_payload().get_password(),
                        connect_packet.get_payload().get_will_topic(),
                        connect_packet.get_payload().get_will_message(),
                        connect_packet.get_connect_flags().get_clean_session(),
                        connect_packet.get_connect_flags().get_will_retain(),
                        connect_packet.get_connect_flags().get_will_flag(),
                        connect_packet.get_connect_flags().get_will_qos1(),
                        connect_packet.get_connect_flags().get_will_qos2(),
                        connect_packet.get_connect_flags().get_username_flag(),
                        connect_packet.get_connect_flags().get_password_flag()
                    );
                    client::update_changes_user(
                        connect_packet.get_payload().get_client_identifier().clone(),
                    );
                    Ok(s)
                }
                Err(_) => {
                    error!("Error al enviar CONNECT");
                    Err(Mqtt5ReturnCodes::MqttRcServerUnavailable)
                }
            };
        }
        Err(_) => {
            error!("Error al crear CONNECT");
            Err(Mqtt5ReturnCodes::MqttRcMalformedPacket)
        }
    }
}

pub fn send_disconnect(stream: &mut TcpStream) -> Result<usize, Mqtt5ReturnCodes> {
    let disc = serializer::new_disconnect();

    return match stream.write(&disc.get_data()) {
        Ok(s) => {
            info!("Desconectado");
            Ok(s)
        }
        Err(_) => {
            error!("error al desconectar");
            Err(Mqtt5ReturnCodes::MqttRcServerUnavailable)
        }
    };
}

pub fn send_pingreq(stream: &mut TcpStream) -> Result<usize, Mqtt5ReturnCodes> {
    let pingreq = serializer::new_pingreq();

    return match stream.write(&pingreq.get_data()) {
        Ok(s) => {
            info!("Paquete PINGREQ enviado.");
            Ok(s)
        }
        Err(_) => {
            error!("Error al enviar PINGREQ.");
            Err(Mqtt5ReturnCodes::MqttRcServerUnavailable)
        }
    };
}
