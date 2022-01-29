use crate::constants_and_structs::mqtt_constants::{
    PacketType, UNSUBACK_PACKET_FLAGS, UNSUBACK_REMAINING_LENGTH,
};

pub struct Unsuback {
    //packet_type: PacketType,
    //unsuback_packet_flags: u8,
    //packet_identifier_msb: u8,
    //packet_identifier_lsb: u8,
    //remaining_length: u8,
    data: Vec<u8>,
}

impl Unsuback {
    pub(crate) fn new() -> Self {
        Unsuback {
            //packet_type: PacketType::UNSUBACK,
            //remaining_length: UNSUBACK_REMAINING_LENGTH,
            //unsuback_packet_flags: UNSUBACK_PACKET_FLAGS,
            //packet_identifier_msb: 0,
            //packet_identifier_lsb: 0,
            data: vec![
                (PacketType::UNSUBACK as u8) << 4 | UNSUBACK_PACKET_FLAGS,
                UNSUBACK_REMAINING_LENGTH,
                0,
                0,
            ],
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}
