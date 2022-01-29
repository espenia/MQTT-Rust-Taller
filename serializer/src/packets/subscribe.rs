use crate::constants_and_structs::mqtt_constants::{PacketType, SUBSCRIBE_PACKET_FLAGS};
use crate::constants_and_structs::topic_filter::TopicFilter;
use crate::mqtt_response::Mqtt5ReturnCodes;
use crate::mqtt_response::MqttError;
use std::error::Error;
use tracing::error;

pub struct Subscribe {
    //packet_type: PacketType,
    //subscribe_packet_flag: u8,
    remaining_length: u8,
    topic_filters: Vec<TopicFilter>,
    data: Vec<u8>,
}

impl Subscribe {
    pub(crate) fn new(topic_filters: &mut Vec<TopicFilter>) -> Result<Self, Box<dyn Error>> {
        let mut subscribe = Subscribe {
            //packet_type: PacketType::SUBSCRIBE,
            //subscribe_packet_flag: SUBSCRIBE_PACKET_FLAGS,
            remaining_length: 0,
            topic_filters: topic_filters.clone(),
            data: vec![(PacketType::SUBSCRIBE as u8) << 4 | SUBSCRIBE_PACKET_FLAGS],
        };

        let mut size = 0;
        let mut total_size = 0;
        for topic_filter in topic_filters.iter_mut() {
            size = topic_filter.get_filter().len();
            if topic_filter.get_length() + 2 != size as u8 {
                error!("[Serializer:Subscribe] Invalid topic filter size");
                return Err(Box::new(MqttError {
                    error: Mqtt5ReturnCodes::MqttPacketInvalidSize,
                }));
            }
            if topic_filter.get_qos() > 2 {
                error!("[Serializer:Subscribe] Invalid Qos");
                return Err(Box::new(MqttError {
                    error: Mqtt5ReturnCodes::MqttRcProtocolError,
                }));
            }
            total_size += size + 1;
        }
        if size == 0 {
            error!("[Serializer:Subscribe] Invalid topic filter size");
            return Err(Box::new(MqttError {
                error: Mqtt5ReturnCodes::MqttRcProtocolError,
            }));
        }
        subscribe.remaining_length = (total_size + 2) as u8; // +2 PACKET IDENTIFIER
        subscribe.data.push(subscribe.remaining_length);
        //PACKET IDENTIFIER IGNORE
        subscribe.data.push(0);
        subscribe.data.push(0);
        for topic_filter in topic_filters.iter_mut() {
            subscribe
                .data
                .append(&mut topic_filter.get_filter().clone());
            subscribe.data.push(topic_filter.get_qos());
        }
        Ok(subscribe)
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    pub fn get_topics(&self) -> Vec<TopicFilter> {
        self.topic_filters.clone()
    }
}
