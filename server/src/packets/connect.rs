use crate::json_helper::{
    read_q_messages, read_topic_subs, read_user_db, read_users, write_q_messages,
    write_topic_unsubs, write_users,
};
use crate::packets::publish::send_queue_messages;
use serializer::mqtt_response::MqttError;
use serializer::{
    new_connack, new_connect_return_code, Connect, ConnectAcknowledgeFlags, ConnectReturnCode,
    Mqtt5ReturnCodes,
};
use std::error::Error;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use tracing::{error, info};

/// Control de logica de paquete Connect
pub fn resolve_connect(
    connect: Connect,
    stream: &mut TcpStream,
    mut user: (u32, String),
    read: &mut TcpStream,
) -> Result<(u32, String), Box<dyn Error>> {
    let flag = connect.get_connect_flags();
    let payload = connect.get_payload();
    let client = payload.get_client_identifier();
    let mut users = read_users()?;
    let user_db = read_user_db()?;
    let mut connect_ack_flags = ConnectAcknowledgeFlags::Sp0;
    let mut return_code: ConnectReturnCode = ConnectReturnCode::ConnectionAccepted;
    let username = payload.get_username();
    let password = payload.get_password();
    let mut keep_alive = payload.get_keep_alive();
    if keep_alive != 0 {
        keep_alive += keep_alive / 2;
        let _result = stream.set_read_timeout(Option::from(Duration::from_secs(keep_alive as u64)));
    }

    info!(
        "Datos del CONNECT: \n \
    Client ID: {:?},\n \
    Username: {:?}, \n \
    Password:{:?}, \n \
    Clean Session: {:?}, \n \
    Will Flag: {:?}, \n \
    Will QoS b1: {:?}, \n \
    Will QoS b2: {:?}, \n \
    Will Retain: {:?}, \n \
    Will Topic: {:?}, \n \
    Will Message: {:?}",
        client,
        username,
        password,
        flag.get_clean_session(),
        flag.get_will_flag(),
        flag.get_will_qos1(),
        flag.get_will_qos2(),
        flag.get_will_retain(),
        payload.get_will_topic(),
        payload.get_will_message()
    );
    user = (user.0, client.clone());
    if flag.get_clean_session() {
        users.insert(user.0, client.clone());
        let mut subs = read_topic_subs()?;
        let mut topics = vec![];
        let mut mess = read_q_messages()?;
        mess.insert(client.clone(), vec![]);
        let keys = subs.keys().clone().collect::<Vec<&String>>();
        let mut mutable_keys = vec![];
        for key in keys {
            mutable_keys.push(key.clone());
        }
        for sub in mutable_keys {
            let mut userqos = subs[&sub].clone();
            let mut pos: i32 = -1;
            for (i, user) in userqos.iter().enumerate() {
                if user.get_user() == client.clone() {
                    pos = i as i32;
                    break;
                }
            }
            if pos != -1 {
                userqos.remove(pos as usize);
                subs.remove(&sub);
                topics.push(sub.clone());
                subs.insert(sub.clone(), userqos);
            }
        }
        if write_q_messages(mess).is_ok() {}
        if write_topic_unsubs(subs, client.clone(), topics).is_ok() {}
    } else {
        if !users.clone().values().any(|v| v.clone() == client.clone()) {
            connect_ack_flags = ConnectAcknowledgeFlags::Sp0;
            return_code = ConnectReturnCode::IdentifierRejected;
            info!(
                "Enviando CONNACK: \n\
            Connect Acknowledge Flags:  {:?}, \n\
            Connect Return Code: {:?}",
                connect_ack_flags, return_code
            );
            return match send_connack(stream, connect_ack_flags, return_code)? {
                true => Ok((0, "".to_string())),
                false => Err(Box::new(MqttError {
                    error: Mqtt5ReturnCodes::MqttRcClientidNotValid,
                })),
            };
        } else {
            for k in users.clone().keys() {
                if users[k] == client.clone() {
                    users.remove(k);
                    break;
                }
            }
            if users.contains_key(&user.0) {
                users.remove(&user.0);
            } else {
            }
            users.insert(user.0, client.clone());
            connect_ack_flags = ConnectAcknowledgeFlags::Sp1;
        }
    }

    if flag.get_password_flag() && flag.get_username_flag() {
        if user_db.contains_key(&username.clone()) {
            if user_db[&username.clone()] == password.clone() {
                return_code = ConnectReturnCode::ConnectionAccepted;
            } else {
                return_code = ConnectReturnCode::BadUserNameOrPassword;
            }
        } else {
            return_code = ConnectReturnCode::BadUserNameOrPassword;
        }
    } else if flag.get_username_flag() && !flag.get_password_flag() {
        if user_db.contains_key(&username.clone()) {
            return_code = ConnectReturnCode::ConnectionAccepted;
        } else {
            return_code = ConnectReturnCode::BadUserNameOrPassword;
        }
    } else if flag.get_password_flag() && !flag.get_username_flag() {
        error!("[CONNECT]flag de password, y no de username");
        return_code = ConnectReturnCode::InvalidProtocol;
    } else if !flag.get_username_flag() && !flag.get_password_flag() {
        return_code = ConnectReturnCode::ConnectionAccepted;
    }

    info!(
        "Enviando CONNACK: \n\
    Connect Acknowledge Flags:  {:?}, \n\
    Connect Return Code: {:?}",
        connect_ack_flags, return_code
    );
    let ret = send_connack(stream, connect_ack_flags, return_code)?;
    if send_queue_messages(stream, user.1.clone(), read).is_ok() {}
    match write_users(users.clone()) {
        Ok(_) => {}
        Err(_) => {
            error!("error al escribir users")
        }
    }
    match ret {
        true => Ok(user),
        false => Err(Box::new(MqttError {
            error: Mqtt5ReturnCodes::MqttRcClientidNotValid,
        })),
    }
}

fn send_connack(
    stream: &mut TcpStream,
    connect_ack_flags: ConnectAcknowledgeFlags,
    return_code: ConnectReturnCode,
) -> Result<bool, Box<dyn Error>> {
    let connect_return_codes = new_connect_return_code(return_code);
    let connack = new_connack(connect_ack_flags, connect_return_codes);
    match stream.write(&connack.get_data()) {
        Ok(_) => Ok(true),
        Err(e) => {
            error!("{}", e.to_string());
            Ok(false)
        }
    }
}
