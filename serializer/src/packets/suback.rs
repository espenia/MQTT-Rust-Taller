use crate::constants_and_structs::mqtt_constants::{
    PacketType, SubackReturnCode, SUBACK_PACKET_FLAGS,
};

pub struct Suback {
    //packet_type: PacketType,
    //suback_packet_flags: u8,
    remaining_length: u8,
    suback_return_codes: Vec<SubackReturnCode>,
    data: Vec<u8>,
}

impl Suback {
    pub(crate) fn new(suback_return_codes: Vec<SubackReturnCode>) -> Self {
        let mut suback = Suback {
            //packet_type: PacketType::SUBACK,
            remaining_length: 0,
            //suback_packet_flags: SUBACK_PACKET_FLAGS,
            suback_return_codes: suback_return_codes.clone(),
            data: vec![(PacketType::SUBACK as u8) << 4 | SUBACK_PACKET_FLAGS],
        };
        suback.remaining_length = (3 * suback_return_codes.len()) as u8;
        suback.data.push(suback.remaining_length);
        for code in suback_return_codes {
            suback.data.push(0);
            suback.data.push(0); //PACKET IDENTIFIER
            suback.data.push(code as u8);
        }
        suback
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    pub fn get_return_codes(&self) -> Vec<SubackReturnCode> {
        self.suback_return_codes.clone()
    }
}
