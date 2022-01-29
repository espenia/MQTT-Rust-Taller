use crate::constants_and_structs::connect_flag::ConnectFlag;
use crate::mqtt_response::{Mqtt5ReturnCodes, MqttError};
use std::error::Error;
use tracing::error;

#[derive(Clone, Debug)]
pub struct PayloadConnect {
    client_identifier: String,
    will_topic: String,
    will_message: String,
    username: String,
    password: String,
    keep_alive: u16,
    data: Vec<u8>,
}

impl PayloadConnect {
    pub(crate) fn new(
        client_identifier: String,
        will_topic: String,
        will_message: String,
        username: String,
        password: String,
        keep_alive: u16,
    ) -> Result<Self, Mqtt5ReturnCodes> {
        let mut payload = PayloadConnect {
            client_identifier: client_identifier.clone(),
            will_topic: will_topic.clone(),
            will_message: will_message.clone(),
            username: username.clone(),
            password: password.clone(),
            keep_alive,
            data: vec![],
        };
        if client_identifier.as_bytes().len() > 25 {
            return Err(Mqtt5ReturnCodes::MqttRcProtocolError);
        }

        if keep_alive > 0xFF {
            payload.data.push(0xFF);
            payload.data.push(0xFF - keep_alive as u8);
        } else {
            payload.data.push(0);
            payload.data.push(keep_alive as u8);
        }
        payload = Self::add_to_payload(client_identifier, payload);
        payload = Self::add_to_payload(will_topic, payload);
        payload = Self::add_to_payload(will_message, payload);
        payload = Self::add_to_payload(username, payload);
        payload = Self::add_to_payload(password, payload);

        Ok(payload)
    }

    fn add_to_payload(array: String, mut payload: PayloadConnect) -> PayloadConnect {
        if !array.is_empty() {
            payload.data.push(0);
            payload.data.push(array.clone().into_bytes().len() as u8);
            payload.data.append(&mut array.into_bytes());
        }
        payload
    }

    pub(crate) fn new_by_hex(
        data: Vec<u8>,
        connect_flags: ConnectFlag,
    ) -> Result<Self, Box<dyn Error>> {
        let mut payload = PayloadConnect {
            client_identifier: "".to_string(),
            will_topic: "".to_string(),
            will_message: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            keep_alive: 0,
            data: data.clone(),
        };

        let mut client_identifier_array: Vec<u8> = vec![];
        let mut will_topic_array: Vec<u8> = vec![];
        let mut will_message_array: Vec<u8> = vec![];
        let mut username_array: Vec<u8> = vec![];
        let mut password_array: Vec<u8> = vec![];
        let mut client_identifier_flag = true;
        let mut will_topic_flag = false;
        let mut will_message_flag = false;
        let mut username_flag = false;
        let mut password_flag = false;
        //let mut keep_alive_flag = false;
        let mut client_identifier_size: usize = 1;
        let mut will_topic_size: usize = 0;
        let mut will_message_size: usize = 0;
        let mut username_size: usize = 0;
        //let mut password_size: usize = 0;
        let mut i: usize = 0;
        while i < data.len() {
            if i == 0 {
                payload.keep_alive = data[i] as u16 + data[i + 1] as u16;
                i += 2;
            }
            if i == 3 {
                client_identifier_size = i + data[i] as usize;
                if client_identifier_size > 23 + 2 {
                    error!("[Serializer:PayloadConnect] Invalid Client Identifier size");
                    return Err((MqttError {
                        error: Mqtt5ReturnCodes::MqttPacketInvalidSize,
                    })
                    .into());
                }
            }
            if i == client_identifier_size {
                client_identifier_array.push(data[i]);
                i += 1;
                if connect_flags.get_will_flag() {
                    will_topic_size = i + data[i + 1] as usize + 1;
                    will_topic_flag = true;
                } else if connect_flags.get_username_flag() {
                    username_size = i + data[i + 1] as usize + 1;
                    username_flag = true;
                } else {
                } // } else{
                  //     keep_alive_flag = true;
                  // }
                client_identifier_flag = false;
            }
            if i == will_topic_size && will_topic_flag {
                will_topic_array.push(data[i]);
                i += 1;
                will_message_size = i + data[i + 1] as usize + 1;
                will_message_flag = true;
                will_topic_flag = false;
            }
            if i == will_message_size && will_message_flag {
                will_message_array.push(data[i]);
                i += 1;
                if connect_flags.get_username_flag() {
                    username_size = i + data[i + 1] as usize + 1;
                    username_flag = true;
                } else if i < data.len() {
                    return Err(Box::new(MqttError {
                        error: Mqtt5ReturnCodes::MqttPacketInvalidSize,
                    }));
                }
                will_message_flag = false;
            }
            if i == username_size && username_flag {
                username_array.push(data[i]);
                i += 1;
                if connect_flags.get_password_flag() {
                    password_flag = true;
                } else if i < data.len() {
                    return Err(Box::new(MqttError {
                        error: Mqtt5ReturnCodes::MqttPacketInvalidSize,
                    }));
                }
                username_flag = false;
            }

            if client_identifier_flag {
                client_identifier_array.push(data[i]);
            }

            if will_topic_flag {
                will_topic_array.push(data[i]);
            }

            if will_message_flag {
                will_message_array.push(data[i]);
            }

            if username_flag {
                username_array.push(data[i]);
            }

            if password_flag {
                password_array.push(data[i])
            }
            i += 1;
        }
        if connect_flags.get_username_flag() {
            payload.username = String::from_utf8(username_array[2..username_array.len()].to_vec())?;
        }
        if connect_flags.get_password_flag() {
            payload.password = String::from_utf8(password_array[2..password_array.len()].to_vec())?;
        }
        if connect_flags.get_will_flag() {
            payload.will_topic =
                String::from_utf8(will_topic_array[2..will_topic_array.len()].to_vec())?;
            payload.will_message =
                String::from_utf8(will_message_array[2..will_message_array.len()].to_vec())?;
        }
        payload.client_identifier =
            String::from_utf8(client_identifier_array[2..client_identifier_array.len()].to_vec())?;
        Ok(payload)
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn get_client_identifier(&self) -> &String {
        &self.client_identifier
    }

    pub fn get_will_topic(&self) -> &String {
        &self.will_topic
    }
    pub fn get_will_message(&self) -> &String {
        &self.will_message
    }

    pub fn get_username(&self) -> &String {
        &self.username
    }
    pub fn get_password(&self) -> &String {
        &self.password
    }
    pub fn get_keep_alive(&self) -> u16 {
        self.keep_alive
    }
}
