#[derive(Copy, Clone, Debug)]
pub struct ConnectFlag {
    byte: u8,
    reserved: bool,      // bit 0
    clean_session: bool, // b1
    will_flag: bool,     // b2
    will_qosb1: bool,    // b3
    will_qosb2: bool,    // b4
    will_retain: bool,   // b5
    password_flag: bool, // b6
    username_flag: bool, // b7
}

use crate::mqtt_response::Mqtt5ReturnCodes;
use crate::tools::converter::{to_bin8, to_hex};

impl ConnectFlag {
    pub(crate) fn new(
        clean_session: Option<bool>,
        will_flag: Option<bool>,
        will_qos1: Option<bool>,
        will_qos2: Option<bool>,
        will_retain: Option<bool>,
        password_flag: Option<bool>,
        username_flag: Option<bool>,
    ) -> Result<Self, Mqtt5ReturnCodes> {
        let connect_flag = ConnectFlag {
            byte: 0,
            reserved: false,
            clean_session: clean_session.unwrap_or_default(),
            will_flag: will_flag.unwrap_or_default(),
            will_qosb1: will_qos1.unwrap_or_default(),
            will_qosb2: will_qos2.unwrap_or_default(),
            will_retain: will_retain.unwrap_or_default(),
            password_flag: password_flag.unwrap_or_default(),
            username_flag: username_flag.unwrap_or_default(),
        };
        if will_qos1.unwrap_or_default() && will_qos2.unwrap_or_default() {
            return Err(Mqtt5ReturnCodes::MqttRcProtocolError);
            // TODO LOGpanic!("invalid protocol, QoS in connect flag cant be 0x03");
        }
        Self::to_hex(connect_flag)
    }

    pub(crate) fn new_by_hex(hex: u8) -> Result<Self, Mqtt5ReturnCodes> {
        let connect_flag = ConnectFlag {
            byte: hex,
            reserved: false,
            clean_session: false,
            will_flag: false,
            will_qosb1: false,
            will_qosb2: false,
            will_retain: false,
            password_flag: false,
            username_flag: false,
        };
        Self::to_bin(connect_flag)
    }

    pub fn get_clean_session(&self) -> bool {
        self.clean_session
    }

    pub fn get_will_flag(&self) -> bool {
        self.will_flag
    }

    pub fn get_will_qos1(&self) -> bool {
        self.will_qosb1
    }

    pub fn get_will_qos2(&self) -> bool {
        self.will_qosb2
    }

    pub fn get_will_retain(&self) -> bool {
        self.will_retain
    }

    pub fn get_password_flag(&self) -> bool {
        self.password_flag
    }

    pub fn get_username_flag(&self) -> bool {
        self.username_flag
    }

    fn to_hex(mut connect_flag: ConnectFlag) -> Result<Self, Mqtt5ReturnCodes> {
        let mut flag = "".to_string();
        if !connect_flag.reserved {
            flag = "0".to_string() + &*flag;
        } else {
            flag = "1".to_string() + &*flag;
        }
        if !connect_flag.clean_session {
            flag = "0".to_string() + &*flag;
        } else {
            flag = "1".to_string() + &*flag;
        }
        if !connect_flag.will_flag {
            flag = "0".to_string() + &*flag;
        } else {
            flag = "1".to_string() + &*flag;
        }
        if !connect_flag.will_qosb1 {
            flag = "0".to_string() + &*flag;
        } else {
            flag = "1".to_string() + &*flag;
        }
        if !connect_flag.will_qosb2 {
            flag = "0".to_string() + &*flag;
        } else {
            flag = "1".to_string() + &*flag;
        }
        if !connect_flag.will_retain {
            flag = "0".to_string() + &*flag;
        } else {
            flag = "1".to_string() + &*flag;
        }
        if !connect_flag.password_flag {
            flag = "0".to_string() + &*flag;
        } else {
            flag = "1".to_string() + &*flag;
        }
        if !connect_flag.username_flag {
            flag = "0".to_string() + &*flag;
        } else {
            flag = "1".to_string() + &*flag;
        }
        let value: i64 = flag.parse().unwrap();
        connect_flag.byte = to_hex(value);
        Ok(connect_flag)
    }

    fn to_bin(mut connect_flag: ConnectFlag) -> Result<Self, Mqtt5ReturnCodes> {
        let bit = to_bin8(connect_flag.byte);
        let bit_array: Vec<char> = bit.chars().collect();
        for (i, item) in bit_array.iter().enumerate() {
            match i {
                0 => {
                    if *item == '0' {
                        connect_flag.username_flag = false;
                    } else {
                        connect_flag.username_flag = true;
                    }
                }
                1 => {
                    if *item == '0' {
                        connect_flag.password_flag = false;
                    } else {
                        connect_flag.password_flag = true;
                    }
                }
                2 => {
                    if *item == '0' {
                        connect_flag.will_retain = false;
                    } else {
                        connect_flag.will_retain = true;
                    }
                }
                3 => {
                    if *item == '0' {
                        connect_flag.will_qosb2 = false;
                    } else {
                        connect_flag.will_qosb2 = true;
                    }
                }
                4 => {
                    if *item == '0' {
                        connect_flag.will_qosb1 = false;
                    } else {
                        connect_flag.will_qosb1 = true;
                    }
                }
                5 => {
                    if *item == '0' {
                        connect_flag.will_flag = false;
                    } else {
                        connect_flag.will_flag = true;
                    }
                }
                6 => {
                    if *item == '0' {
                        connect_flag.clean_session = false;
                    } else {
                        connect_flag.clean_session = true;
                    }
                }
                7 => connect_flag.reserved = false,
                _ => {}
            }
        }
        if connect_flag.will_qosb1 && connect_flag.will_qosb2 {
            return Err(Mqtt5ReturnCodes::MqttRcProtocolError);
            // TODO LOGGERpanic!("invalid protocol, QoS in connect flag cant be 0x03");
        }
        Ok(connect_flag)
    }

    pub(crate) fn hex_value(&self) -> u8 {
        self.byte
    }
}
