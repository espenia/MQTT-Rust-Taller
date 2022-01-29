#[derive(Copy, Clone, Debug)]
pub struct ConnectReturnCodes {
    byte: u8,
    accepted: bool, // bit 0
    reason: u8,
}

use crate::constants_and_structs::mqtt_constants::ConnectReturnCode;
/// 0x00 Connection Accepted
/// 0x01 Connection Refused, unacceptable protocol version
/// 0x02 Connection Refused, identifier rejected
/// 0x03 Connection Refused, Server unavailable
/// 0x04 Connection Refused, bad user name or password
/// 0x05 Connection Regused, not authorized
/// 6-2555 Reserverd for future use.
/// If a server sends a CONNACK packet containing a non-zero return code it MUST close the Network Connection.
use crate::tools::converter::to_bin8;

impl ConnectReturnCodes {
    pub(crate) fn new(rejected_reason: ConnectReturnCode) -> Self {
        let connect_return_codes = ConnectReturnCodes {
            byte: 0,
            accepted: false,
            reason: rejected_reason as u8,
        };
        Self::to_hex(connect_return_codes)
    }

    pub(crate) fn new_by_hex(hex: u8) -> Self {
        let connect_return_codes = ConnectReturnCodes {
            byte: hex,
            accepted: false,
            reason: 0x0,
        };
        Self::to_bin(connect_return_codes)
    }

    fn to_hex(mut connect_return_codes: ConnectReturnCodes) -> Self {
        match connect_return_codes.reason {
            1 => {
                connect_return_codes.byte = 0x11;
                connect_return_codes.accepted = false;
            } // ConnectReturnCode::InvalidProtocol;
            2 => {
                connect_return_codes.byte = 0x12;
                connect_return_codes.accepted = false;
            } // ConnectReturnCode::IdentifierRejected;
            3 => {
                connect_return_codes.byte = 0x13;
                connect_return_codes.accepted = false;
            } // ConnectReturnCode
            4 => {
                connect_return_codes.byte = 0x14;
                connect_return_codes.accepted = false;
            } // ConnectReturnCode::BadUserNameOrPassword;
            5 => {
                connect_return_codes.byte = 0x15;
                connect_return_codes.accepted = false;
            } // ConnectReturnCode::NotAuthorized;
            0 => {
                connect_return_codes.byte = 0x0;
                connect_return_codes.accepted = true;
            } // ConnectReturnCode::ConnectionAccepted;
            _ => {
                connect_return_codes.byte = 0x16;
                connect_return_codes.accepted = false;
            } // ConnectReturnCode::CloseConnection;
        }
        connect_return_codes
    }

    fn to_bin(mut connect_return_codes: ConnectReturnCodes) -> Self {
        let mut bit = to_bin8(connect_return_codes.byte);
        if connect_return_codes.byte != 0 {
            connect_return_codes.accepted = false;
        } else {
            connect_return_codes.accepted = true;
            connect_return_codes.reason = 0x00;
            return connect_return_codes;
        }
        bit.replace_range(..4, "");
        match bit.as_ref() {
            "0001" => connect_return_codes.reason = ConnectReturnCode::InvalidProtocol as u8,
            "0010" => connect_return_codes.reason = ConnectReturnCode::IdentifierRejected as u8,
            "0011" => connect_return_codes.reason = ConnectReturnCode::ServerUnavailable as u8,
            "0100" => connect_return_codes.reason = ConnectReturnCode::BadUserNameOrPassword as u8,
            "0101" => connect_return_codes.reason = ConnectReturnCode::NotAuthorized as u8,
            _ => connect_return_codes.reason = ConnectReturnCode::CloseConnection as u8,
        }
        connect_return_codes
    }

    pub(crate) fn hex_value(self) -> u8 {
        self.byte
    }

    pub fn is_accepted(&self) -> bool {
        self.accepted
    }
    pub fn get_reason(&self) -> u8 {
        self.reason
    }
}
