use crate::constants_and_structs::mqtt_constants::{
    PacketType, PINGREQ_PACKET_FLAGS, PINGREQ_REMAINING_LENGTH,
};

pub struct Pingreq {
    //packet_type: PacketType,
    //pingresq_packet_flags: u8,
    //remaining_length: u8,
    data: Vec<u8>,
}

impl Pingreq {
    pub(crate) fn new() -> Self {
        Pingreq {
            //packet_type: PacketType::PINGREQ,
            //remaining_length: PINGREQ_REMAINING_LENGTH,
            //pingresq_packet_flags: PINGREQ_PACKET_FLAGS,
            data: vec![
                (PacketType::PINGREQ as u8) << 4 | PINGREQ_PACKET_FLAGS,
                PINGREQ_REMAINING_LENGTH,
            ],
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}
