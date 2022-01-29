use crate::constants_and_structs::mqtt_constants::{PacketType, UNSUBSCRIBE_PACKET_FLAGS};
use crate::constants_and_structs::topic_filter::TopicFilter;
use crate::mqtt_response::{Mqtt5ReturnCodes, MqttError};
use std::error::Error;
use tracing::error;

pub struct Unsubscribe {
    //packet_type: PacketType,
    //unsubscribe_packet_flag: u8,
    remaining_length: u8,
    topic_filters: Vec<TopicFilter>,
    data: Vec<u8>,
}

impl Unsubscribe {
    pub(crate) fn new(topic_filters: &mut Vec<TopicFilter>) -> Result<Self, Box<dyn Error>> {
        let mut unsuscribe = Unsubscribe {
            //packet_type: PacketType::UNSUSCRIBE,
            //unsubscribe_packet_flag: UNSUBSCRIBE_PACKET_FLAGS,
            remaining_length: 0,
            topic_filters: topic_filters.clone(),
            data: vec![(PacketType::UNSUSCRIBE as u8) << 4 | UNSUBSCRIBE_PACKET_FLAGS],
        };
        let mut remaining_length = 0;
        for topic_filter in topic_filters.iter_mut() {
            let size = topic_filter.get_filter().len();
            remaining_length += size;
            if topic_filter.get_length() + 2 != size as u8 {
                error!("[Serializer:Unsubscribe] Invalid topic filter size");
                return Err(Box::new(MqttError {
                    error: Mqtt5ReturnCodes::MqttPacketInvalidSize,
                }));
            }
        }
        unsuscribe.remaining_length = (remaining_length + 2) as u8; //PACKET IDENTIFIER
        unsuscribe.data.push((remaining_length + 2) as u8);
        unsuscribe.data.push(0);
        unsuscribe.data.push(0);
        for topic_filter in topic_filters.iter_mut() {
            unsuscribe
                .data
                .append(&mut topic_filter.get_filter().clone())
        }
        Ok(unsuscribe)
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    pub fn get_topic_filters(&self) -> Vec<TopicFilter> {
        self.topic_filters.clone()
    }
}
