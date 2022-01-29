use crate::json_helper;
use crate::json_helper::{
    read_q_messages, read_topic_subs, write_remove_q_messages, write_retain_messages,
    write_topic_subs,
};
use crate::socket::Socket;
use serializer::mqtt_response::MqttError;
use serializer::{
    new_mqtt_header, new_publish, new_publish_by_hex, new_publish_packet_flags, new_topic_filter,
    Mqtt5ReturnCodes, PacketType, Publish, PublishFlag, TopicFilter,
};
use std::error::Error;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use tracing::{error, info, warn};

/// Logica de paquete Publish
pub fn resolve_publish(
    publish: serializer::Publish,
    stream: &mut TcpStream,
    sender: &Sender<Publish>,
) -> Result<bool, Box<dyn Error>> {
    let flags = publish.get_flags();
    let topic = publish.get_topic();

    write_retain(flags, topic.clone(), publish.clone());

    let mut subs = read_topic_subs()?;
    subs.entry(topic.get_topic()).or_insert_with(|| Vec::new());
    match write_topic_subs(subs) {
        Ok(_) => {}
        Err(_) => {
            error!("error al escribir topic subs")
        }
    }
    match flags.get_qos() {
        3 => {
            error!("QOS no valido para Publish")
        }
        1 => send_puback(stream),
        _ => {}
    }
    let ret = send_message_to_subs(sender, publish)?;

    Ok(ret)
}

fn send_puback(stream: &mut TcpStream) {
    let puback = serializer::new_puback();
    info!("Enviando PUBACK: {:?}", puback.get_data());
    match stream.write(&puback.get_data()) {
        Ok(_) => {}
        Err(_) => {
            error!("error al mandar suback")
        }
    }
}

pub fn write_retain(flags: PublishFlag, topic: TopicFilter, publish: Publish) {
    if flags.get_retain() {
        let mut retain_message = json_helper::read_retain_messages().ok().unwrap();
        if retain_message.contains_key(&topic.get_topic()) {
            let mut value = retain_message[(&topic.get_topic())].clone();
            value.push(publish.get_payload());
            retain_message.insert(topic.get_topic(), value);
        } else {
            retain_message.insert(topic.get_topic(), vec![publish.get_payload()]);
        }
        match json_helper::write_retain_messages(retain_message) {
            Ok(_) => {}
            Err(_) => {
                error!("error al escribir retain messages")
            }
        }
    }
}

pub fn send_retain_messages_to_sub(
    topic: String,
    stream: &mut TcpStream,
    user: String,
) -> Result<bool, Box<dyn Error>> {
    let mut retain_message = json_helper::read_retain_messages()?;
    let subs = json_helper::read_topic_subs()?;
    if !retain_message.contains_key(&topic) {
        retain_message.insert(topic.clone(), vec![]);
    }
    let messages = retain_message[&topic].clone();
    let user_qos = subs[&topic].clone();

    let tf: TopicFilter;
    match new_topic_filter(topic) {
        Ok(t) => tf = t,
        Err(e) => return Err(Box::new(MqttError { error: e })),
    }

    let mut publish_vec: Vec<Publish> = vec![];
    for userqos in user_qos {
        if userqos.get_user() == user {
            let p_flags = set_flag(userqos.get_qos())?;
            for message in messages.clone() {
                match new_publish(p_flags, tf.clone(), message) {
                    Ok(p) => publish_vec.push(p),
                    Err(e) => return Err(Box::new(MqttError { error: e })),
                }
            }
            break;
        }
    }

    for publ in publish_vec {
        match stream.write(&publ.get_data()) {
            Ok(_) => {
                if publ.get_flags().get_qos() == 1 {
                    //TODO puback
                }
            }
            Err(e) => {
                error!(
                    "[Server:Publish] cuando enviando publish a subscriptor {:?}",
                    e.to_string()
                )
            }
        }
    }

    match write_retain_messages(retain_message) {
        Ok(_) => {}
        Err(e) => {
            error!(
                "[Server:Publish] cuando escribiendo retain messages {:?}",
                e.to_string()
            )
        }
    }

    Ok(true)
}

fn send_message_to_subs(
    sender: &Sender<Publish>,
    publish: Publish,
) -> Result<bool, Box<dyn Error>> {
    match sender.send(publish) {
        Ok(_) => {}
        Err(e) => {
            error!(
                "[Server:Publish] cuando enviando publish a subscriptor {:?}",
                e.to_string()
            )
        }
    }
    Ok(true)
}

pub fn send_queue_messages(
    stream: &mut TcpStream,
    user: String,
    read: &mut TcpStream,
) -> Result<bool, Box<dyn Error>> {
    let mut mess_hash = read_q_messages()?;
    if mess_hash.contains_key(&user) {
        let messages = mess_hash[&user].clone();
        for message in messages {
            let header;
            match new_mqtt_header(message) {
                Ok(h) => header = h,
                Err(e) => return Err(Box::new(MqttError { error: e })),
            }
            let publish = new_publish_by_hex(header)?;
            match stream.write(&publish.get_data()) {
                Ok(_) => {
                    if publish.get_flags().get_qos() == 1 {
                        match Socket::read_all(read)?.get_control_packet_type() {
                            PacketType::PUBACK => {}
                            _ => {
                                warn!(
                                        "[Server:Publish] packet invalido cuando enviando publish a subscriptor");
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "[Server:Publish] cuando enviando publish a subscriptor {:?}",
                        e.to_string()
                    )
                }
            }
        }
        mess_hash.remove(&user);
        mess_hash.insert(user.clone(), vec![]);
        if write_remove_q_messages(mess_hash, user).is_ok() {}
        Ok(true)
    } else {
        error!("User not found in hashmap");
        Err(Box::new(MqttError {
            error: Mqtt5ReturnCodes::MqttRcNoMatchingSubscribers,
        }))
    }
}

fn set_flag(qos: u8) -> Result<PublishFlag, Box<dyn Error>> {
    let p_flags: PublishFlag;
    match qos {
        0 => match new_publish_packet_flags(None, None, None, None) {
            Ok(p) => p_flags = p,
            Err(e) => return Err(Box::new(MqttError { error: e })),
        },
        1 => match new_publish_packet_flags(None, None, Option::from(true), None) {
            Ok(p) => p_flags = p,
            Err(e) => return Err(Box::new(MqttError { error: e })),
        },
        _ => {
            return Err(Box::new(MqttError {
                error: Mqtt5ReturnCodes::MqttRcTopicFilterInvalid,
            }))
        }
    }
    Ok(p_flags)
}
