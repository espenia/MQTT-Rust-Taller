use crate::constants_and_structs::connect_flag::ConnectFlag;
use crate::constants_and_structs::mqtt_constants::{
    PacketType, LENGTH_LSB_CONNECT, LENGTH_MSB_CONNECT, PACKET_FLAGS_CONNECT, PROTOCOL_NAME_M,
    PROTOCOL_NAME_Q, PROTOCOL_NAME_T, PROTOCOL_VERSION,
};
use crate::constants_and_structs::payload_connect::PayloadConnect;
use crate::mqtt_response::Mqtt5ReturnCodes;
use crate::mqtt_response::MqttError;
use std::error::Error;
use tracing::error;

#[derive(Debug, Clone)]
pub struct Connect {
    packet_type: PacketType,
    connect_packet_flags: u8,
    remaining_length: u8,
    lsb: u8,
    msb: u8,
    m: u8,
    q: u8,
    t: u8,
    protocol_version: u8,
    connect_flag: ConnectFlag,
    payload: PayloadConnect,
    data: Vec<u8>,
}

impl Connect {
    pub(crate) fn new(
        connect_flag: ConnectFlag,
        payload_connect: PayloadConnect,
    ) -> Result<Self, Box<dyn Error>> {
        let mut connect = Connect {
            packet_type: PacketType::CONNECT,
            connect_packet_flags: PACKET_FLAGS_CONNECT,
            remaining_length: 0,
            lsb: LENGTH_LSB_CONNECT,
            msb: LENGTH_MSB_CONNECT,
            m: PROTOCOL_NAME_M,
            q: PROTOCOL_NAME_Q,
            t: PROTOCOL_NAME_T,
            protocol_version: PROTOCOL_VERSION,
            connect_flag,
            payload: payload_connect.clone(),
            data: vec![],
        };
        let payload = &mut payload_connect.get_data().clone();
        let remaining_length = 8 + payload.len();
        connect.remaining_length = remaining_length as u8;
        connect.data = vec![
            (PacketType::CONNECT as u8) << 4 | PACKET_FLAGS_CONNECT,
            remaining_length as u8,
            LENGTH_LSB_CONNECT,
            LENGTH_MSB_CONNECT,
            PROTOCOL_NAME_M,
            PROTOCOL_NAME_Q,
            PROTOCOL_NAME_T,
            PROTOCOL_NAME_T,
            PROTOCOL_VERSION,
            connect_flag.hex_value(),
        ];
        if payload_connect.get_client_identifier().is_empty() && !connect_flag.get_clean_session() {
            error!("[Serializer:Connect] No valid Client identifier");
            return Err(Box::new(MqttError {
                error: Mqtt5ReturnCodes::MqttRcProtocolError,
            }));
        }
        if (!payload_connect.get_will_topic().is_empty()
            || !payload_connect.get_will_message().is_empty())
            && !connect_flag.get_will_flag()
        {
            error!("[Serializer:Connect] Will values are not empty but will flag is 0");
            return Err(Box::new(MqttError {
                error: Mqtt5ReturnCodes::MqttRcProtocolError,
            }));
        }

        if !payload_connect.get_username().is_empty() && !connect_flag.get_username_flag() {
            error!("[Serializer:Connect] username is not empty but username flag is 0");
            return Err(Box::new(MqttError {
                error: Mqtt5ReturnCodes::MqttRcProtocolError,
            }));
        }

        if !payload_connect.get_password().is_empty() && !connect_flag.get_password_flag() {
            error!("[Serializer:Connect] pasword is not empty but password flag is 0");
            return Err(Box::new(MqttError {
                error: Mqtt5ReturnCodes::MqttRcProtocolError,
            }));
        }

        connect.data.append(payload);
        Ok(connect)
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    pub fn get_connect_flags(&self) -> ConnectFlag {
        self.connect_flag
    }

    pub fn get_payload(&self) -> PayloadConnect {
        self.payload.clone()
    }
}
