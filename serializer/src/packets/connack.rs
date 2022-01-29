use crate::constants_and_structs::mqtt_constants::{
    ConnectAcknowledgeFlags, PacketType, PACKET_FLAGS_CONNACK, REMAINING_LENGTH_CONNACK,
};
use crate::ConnectReturnCodes;

pub struct Connack {
    packet_type: PacketType,
    connack_packet_flags: u8,
    remaining_length: u8,
    connect_acknowledge_flags: ConnectAcknowledgeFlags,
    connect_return_codes: ConnectReturnCodes,
    data: Vec<u8>,
}

impl Connack {
    pub(crate) fn new(
        connect_acknowledge_flags: ConnectAcknowledgeFlags,
        connect_return_codes: ConnectReturnCodes,
    ) -> Self {
        Connack {
            packet_type: PacketType::CONNACK,
            connack_packet_flags: PACKET_FLAGS_CONNACK,
            remaining_length: REMAINING_LENGTH_CONNACK,
            connect_acknowledge_flags,
            connect_return_codes,
            data: vec![
                ((PacketType::CONNACK as u8) << 4) | PACKET_FLAGS_CONNACK,
                REMAINING_LENGTH_CONNACK,
                connect_acknowledge_flags as u8,
                connect_return_codes.hex_value(),
            ],
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    pub fn get_packet_type(&self) -> PacketType {
        self.packet_type
    }
    pub fn get_connack_packet_flags(&self) -> u8 {
        self.connack_packet_flags
    }
    pub fn get_remaining_length(&self) -> u8 {
        self.remaining_length
    }
    pub fn get_connect_acknowledge_flags(&self) -> ConnectAcknowledgeFlags {
        self.connect_acknowledge_flags
    }
    pub fn get_connect_return_codes(&self) -> ConnectReturnCodes {
        self.connect_return_codes
    }
}
