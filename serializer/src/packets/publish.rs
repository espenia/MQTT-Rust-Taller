use crate::constants_and_structs::mqtt_constants::PacketType;

use crate::constants_and_structs::publish_flag::PublishFlag;
use crate::constants_and_structs::topic_filter::TopicFilter;
use crate::mqtt_response::Mqtt5ReturnCodes;

#[derive(Clone)]
pub struct Publish {
    packet_type: PacketType,
    publish_packet_flags: PublishFlag,
    remaining_length: u8,
    topic_filter: TopicFilter,
    pmsb: u8,
    plsb: u8,
    payload: String,
    data: Vec<u8>,
}

impl Publish {
    pub(crate) fn new(
        publish_packet_flags: PublishFlag,
        publish_topic: TopicFilter,
        payload: String,
    ) -> Result<Self, Mqtt5ReturnCodes> {
        let mut publish = Publish {
            packet_type: PacketType::PUBLISH,
            publish_packet_flags,
            remaining_length: 0,
            topic_filter: publish_topic.clone(),
            pmsb: 0,
            plsb: 0,
            payload: payload.clone(),
            data: vec![(PacketType::PUBLISH as u8) << 4 | publish_packet_flags.hex_value()],
        };
        let mut filter = publish_topic.get_filter().clone();

        publish.pmsb = 0;
        publish.plsb = (filter.len() - 2) as u8;
        if publish.publish_packet_flags.get_qos() == 1 {
            publish.remaining_length =
                (filter.len() + payload.clone().into_bytes().len() + 2) as u8; //PACKET IDENTIFIER + 2
        } else {
            publish.remaining_length = (filter.len() + payload.clone().into_bytes().len()) as u8;
        }
        publish.data.push(publish.remaining_length);
        publish.data.append(&mut filter);
        if publish.publish_packet_flags.get_qos() == 1 {
            publish.data.push(0); //PACKET IDENTIFIER
            publish.data.push(0);
        }
        publish.data.append(&mut payload.into_bytes());
        Ok(publish)
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    pub fn get_flags(&self) -> PublishFlag {
        self.publish_packet_flags
    }
    pub fn get_topic(&self) -> TopicFilter {
        self.topic_filter.clone()
    }
    pub fn get_payload(&self) -> String {
        self.payload.clone()
    }
    pub fn set_qos_flag(&mut self, qos: u8) -> Self {
        self.publish_packet_flags = self.publish_packet_flags.set_qos(qos);
        self.clone()
    }
}
