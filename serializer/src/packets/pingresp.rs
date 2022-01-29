use crate::constants_and_structs::mqtt_constants::{
    PacketType, PINGRESP_PACKET_FLAGS, PINGRESP_REMAINING_LENGTH,
};

pub struct Pingresp {
    //packet_type: PacketType,
    //pingresp_packet_flags: u8,
    //remaining_length: u8,
    data: Vec<u8>,
}

impl Pingresp {
    pub(crate) fn new() -> Self {
        Pingresp {
            //packet_type: PacketType::PINGRESP,
            //remaining_length: PINGRESP_REMAINING_LENGTH,
            //pingresp_packet_flags: PINGRESP_PACKET_FLAGS,
            data: vec![
                (PacketType::PINGRESP as u8) << 4 | PINGRESP_PACKET_FLAGS,
                PINGRESP_REMAINING_LENGTH,
            ],
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}
