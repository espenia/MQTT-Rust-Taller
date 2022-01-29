use crate::constants_and_structs::mqtt_constants::{
    PacketType, DISCONNECT_PACKET_FLAGS, DISCONNECT_REMAINING_LENGTH,
};

pub struct Disconnect {
    //packet_type: PacketType,
    //disconnect_packet_flags: u8,
    //remaining_length: u8,
    data: Vec<u8>,
}

impl Disconnect {
    pub(crate) fn new() -> Self {
        Disconnect {
            //packet_type: PacketType::DISCONNECT,
            //remaining_length: DISCONNECT_REMAINING_LENGTH,
            //disconnect_packet_flags: DISCONNECT_PACKET_FLAGS,
            data: vec![
                (PacketType::DISCONNECT as u8) << 4 | DISCONNECT_PACKET_FLAGS,
                DISCONNECT_REMAINING_LENGTH,
            ],
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}
