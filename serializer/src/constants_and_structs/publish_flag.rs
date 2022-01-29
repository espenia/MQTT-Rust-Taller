#[derive(Copy, Clone)]
pub struct PublishFlag {
    byte: u8,
    retain: bool,   // bit 0
    qosb1: bool,    // b1
    qosb2: bool,    // b2
    dup_flag: bool, // b3
}

use crate::mqtt_response::Mqtt5ReturnCodes;
use crate::tools::converter::{to_bin8, to_hex};
use tracing::error;

impl PublishFlag {
    pub(crate) fn new(
        retain: Option<bool>,
        qos1: Option<bool>,
        qos2: Option<bool>,
        dup_flag: Option<bool>,
    ) -> Result<Self, Mqtt5ReturnCodes> {
        let publish_flag = PublishFlag {
            byte: 0,
            retain: retain.unwrap_or_default(),
            qosb1: qos1.unwrap_or_default(),
            qosb2: qos2.unwrap_or_default(),
            dup_flag: dup_flag.unwrap_or_default(),
        };
        Self::to_hex(publish_flag)
    }

    pub(crate) fn new_by_hex(hex: u8) -> Result<Self, Mqtt5ReturnCodes> {
        let publish_flag = PublishFlag {
            byte: hex,
            retain: false,
            qosb1: false,
            qosb2: false,
            dup_flag: false,
        };
        Self::to_bin(publish_flag)
    }
    fn to_hex(mut publish_flag: PublishFlag) -> Result<Self, Mqtt5ReturnCodes> {
        let mut flag = "".to_string();
        if !publish_flag.retain {
            flag = "0".to_string() + &*flag;
        } else {
            flag = "1".to_string() + &*flag;
        }
        if publish_flag.qosb1 && publish_flag.qosb2 {
            error!("[Serializer:PublishFlag] Invalid qos");
            return Err(Mqtt5ReturnCodes::MqttRcProtocolError);
        }
        if !publish_flag.qosb1 {
            flag = "0".to_string() + &*flag;
        } else {
            flag = "1".to_string() + &*flag;
        }
        if !publish_flag.qosb2 {
            flag = "0".to_string() + &*flag;
        } else {
            flag = "1".to_string() + &*flag;
        }
        if !publish_flag.dup_flag {
            flag = "0".to_string() + &*flag;
        } else {
            flag = "1".to_string() + &*flag;
        }
        let value: i64 = flag.parse().unwrap();
        publish_flag.byte = to_hex(value);
        Ok(publish_flag)
    }

    fn to_bin(mut publish_flag: PublishFlag) -> Result<Self, Mqtt5ReturnCodes> {
        let bit = to_bin8(publish_flag.byte);
        let bit_array: Vec<char> = bit.chars().collect();
        if bit_array[1] == '1' && bit_array[2] == '1' {
            error!("[Serializer:PublishFlag] Invalid qos");
            return Err(Mqtt5ReturnCodes::MqttRcProtocolError);
        }
        for (i, item) in bit_array.iter().enumerate() {
            match i {
                7 => {
                    if *item == '0' {
                        publish_flag.retain = false;
                    } else {
                        publish_flag.retain = true;
                    }
                }
                6 => {
                    if *item == '0' {
                        publish_flag.qosb1 = false;
                    } else {
                        publish_flag.qosb1 = true;
                    }
                }
                5 => {
                    if *item == '0' {
                        publish_flag.qosb2 = false;
                    } else {
                        publish_flag.qosb2 = true;
                    }
                }
                4 => {
                    if *item == '0' {
                        publish_flag.dup_flag = false;
                    } else {
                        publish_flag.dup_flag = true;
                    }
                }
                _ => {}
            }
        }
        Ok(publish_flag)
    }

    pub fn set_qos(&mut self, qos: u8) -> Self {
        match qos {
            0 => {
                self.qosb2 = false;
                self.qosb1 = false;
                if let Ok(p) = Self::to_hex(*self) {
                    return p;
                }
            }
            1 => {
                self.qosb1 = true;
                self.qosb2 = false;
                if let Ok(p) = Self::to_hex(*self) {
                    return p;
                }
            }
            _ => {}
        }
        *self
    }

    pub fn hex_value(&self) -> u8 {
        self.byte
    }

    pub fn get_retain(&self) -> bool {
        self.retain
    }

    pub fn get_qos(&self) -> u8 {
        return if !self.qosb1 && self.qosb2 {
            2
        } else if !self.qosb2 && !self.qosb1 {
            0
        } else if self.qosb1 && !self.qosb2 {
            1
        } else {
            error!("[Serializer:PublishFlag] Invalid qos");
            2
        };
    }
}
