use crate::constants_and_structs::mqtt_constants::{
    PacketType, PUBACK_PACKET_FLAGS, PUBACK_REMAINING_LENGTH,
};

pub struct Puback {
    //packet_type: PacketType,
    //puback_packet_flags: u8,
    //remaining_length: u8,
    //pmsb: u8,
    //plsb: u8,
    data: Vec<u8>,
}

impl Puback {
    pub(crate) fn new() -> Self {
        Puback {
            //packet_type: PacketType::PUBACK,
            //remaining_length: PUBACK_REMAINING_LENGTH,
            //pmsb: 0,
            //plsb: 0,
            //puback_packet_flags: 0,
            data: vec![
                (PacketType::PUBACK as u8) << 4 | PUBACK_PACKET_FLAGS,
                PUBACK_REMAINING_LENGTH,
                0,
                0,
            ],
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}
