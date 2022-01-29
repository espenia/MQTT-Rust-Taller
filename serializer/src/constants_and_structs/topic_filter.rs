use crate::mqtt_response::Mqtt5ReturnCodes::MqttPacketInvalidSize;
use crate::mqtt_response::{Mqtt5ReturnCodes, MqttError};
use std::error::Error;
use tracing::error;

#[derive(Clone)]
pub struct TopicFilter {
    topic: String,
    length_msb: u8,
    length_lsb: u8,
    filter: Vec<u8>,
    qos: u8,
}
impl TopicFilter {
    pub(crate) fn new(
        length_msb: u8,
        length_lsb: u8,
        filter: Vec<u8>,
        qos: Option<u8>,
        topic: String,
    ) -> Result<Self, Mqtt5ReturnCodes> {
        if length_lsb + 2 != (filter.len()) as u8 {
            error!("[Serializer:TopicFilter] Invalid topic size");
            return Err(Mqtt5ReturnCodes::MqttPacketInvalidSize);
        }
        let str = String::from_utf8(filter.get(2..).unwrap().to_vec());
        if str.is_err() {
            error!("[Serializer:TopicFilter] Invalid topic filter");
            return Err(Mqtt5ReturnCodes::MqttRcTopicFilterInvalid);
        }
        if topic != str.unwrap() {
            error!("[Serializer:TopicFilter] Invalid topic filter");
            return Err(Mqtt5ReturnCodes::MqttRcTopicNameInvalid);
        }

        if qos.is_some() && qos.unwrap() > 2 {
            error!("[Serializer:TopicFilter] Invalid topic qos");
            return Err(Mqtt5ReturnCodes::MqttRcProtocolError);
        }

        Ok(TopicFilter {
            length_msb,
            length_lsb,
            filter,
            qos: qos.unwrap_or(3),
            topic,
        })
    }

    pub(crate) fn new_by_hex(
        length_msb: u8,
        length_lsb: u8,
        filter: Vec<u8>,
        qos: Option<u8>,
    ) -> Result<Self, Box<dyn Error>> {
        if length_lsb + 2 != (filter.len()) as u8 {
            error!("[Serializer:TopicFilter] Invalid topic size");
            return Err(Box::new(MqttError {
                error: MqttPacketInvalidSize,
            }));
        }
        let mut filter_topic = vec![];
        for item in filter.iter().skip(2) {
            filter_topic.push(*item);
        }

        let str = String::from_utf8(filter_topic.clone());
        if str.is_err() {
            error!("[Serializer:TopicFilter] Invalid topic");
            return Err(Box::new(MqttError {
                error: Mqtt5ReturnCodes::MqttRcTopicFilterInvalid,
            }));
        }
        if qos.is_some() && qos.unwrap() > 2 {
            error!("[Serializer:TopicFilter] Invalid topic qos");
            return Err(Box::new(MqttError {
                error: Mqtt5ReturnCodes::MqttRcProtocolError,
            }));
        }

        Ok(TopicFilter {
            length_msb,
            length_lsb,
            filter,
            qos: qos.unwrap_or(3),
            topic: str.unwrap(),
        })
    }

    pub fn get_filter(&self) -> &Vec<u8> {
        &self.filter
    }
    pub fn get_length(&self) -> u8 {
        self.length_lsb
    }
    pub fn get_qos(&self) -> u8 {
        self.qos
    }
    pub fn get_topic(&self) -> String {
        self.topic.clone()
    }
}
