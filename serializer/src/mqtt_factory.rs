use crate::constants_and_structs::connect_flag::ConnectFlag;
use crate::constants_and_structs::connect_return_codes::ConnectReturnCodes;
use crate::constants_and_structs::mqtt_constants::{
    ConnectAcknowledgeFlags, PacketType, SubackReturnCode,
};
use crate::constants_and_structs::payload_connect::PayloadConnect;
use crate::constants_and_structs::publish_flag::PublishFlag;
use crate::constants_and_structs::topic_filter::TopicFilter;
use crate::mqtt_response::{Mqtt5ReturnCodes, MqttError};
use crate::packets::connack::Connack;
use crate::packets::connect::Connect;
use crate::packets::disconnect::Disconnect;
use crate::packets::pingreq::Pingreq;
use crate::packets::pingresp::Pingresp;
use crate::packets::publish::Publish;
use crate::packets::suback::Suback;
use crate::packets::subscribe::Subscribe;
use crate::packets::unsuback::Unsuback;
use crate::packets::unsubscribe::Unsubscribe;
use crate::tools::converter::to_bin4;
use std::error::Error;
use std::option::Option::None;
use tracing::error;

#[derive(Debug)]
pub struct MqttHeader {
    control_packet_type: PacketType,
    remaining_length: u8,
    data: Vec<u8>,
}

impl MqttHeader {
    pub(crate) fn new(data: Vec<u8>) -> Result<Self, Mqtt5ReturnCodes> {
        let ret = MqttHeader {
            control_packet_type: PacketType::CONNECT,
            remaining_length: 0,
            data,
        };
        check_packet_type(ret)
    }

    pub fn get_control_packet_type(&self) -> PacketType {
        self.control_packet_type
    }
    pub fn get_remaining_length(&self) -> u8 {
        self.remaining_length
    }
}

fn check_packet_type(mut header: MqttHeader) -> Result<MqttHeader, Mqtt5ReturnCodes> {
    if header.data.is_empty() {
        error!("[Serializer:Mqtt Factory] No valid header");
        return Err(Mqtt5ReturnCodes::MqttRcProtocolError);
    }

    let bin = to_bin4(header.data[0] >> 4);
    match bin.as_ref() {
        "0001" => header.control_packet_type = PacketType::CONNECT,
        "0010" => header.control_packet_type = PacketType::CONNACK,
        "0011" => header.control_packet_type = PacketType::PUBLISH,
        "0100" => header.control_packet_type = PacketType::PUBACK,
        "0101" => header.control_packet_type = PacketType::PUBREL,
        "0110" => header.control_packet_type = PacketType::PUBREC,
        "0111" => header.control_packet_type = PacketType::PUBCOMP,
        "1000" => header.control_packet_type = PacketType::SUBSCRIBE,
        "1001" => header.control_packet_type = PacketType::SUBACK,
        "1010" => header.control_packet_type = PacketType::UNSUSCRIBE,
        "1011" => header.control_packet_type = PacketType::UNSUBACK,
        "1100" => header.control_packet_type = PacketType::PINGREQ,
        "1101" => header.control_packet_type = PacketType::PINGRESP,
        "1110" => header.control_packet_type = PacketType::DISCONNECT,
        _ => {
            error!("[Serializer:Mqtt Factory] No valid header");
            return Err(Mqtt5ReturnCodes::MqttRcProtocolError);
        }
    }
    header.remaining_length = header.data[1];
    Ok(header)
}

pub(crate) fn new_connect(header: MqttHeader) -> Result<Connect, Box<dyn Error>> {
    if header.data.len() > 2 {
        if (header.data[1] + 2) as usize != header.data.len() {
            error!("[Serializer:MqttFactory] Invalid Connect size");
            return Err(Box::new(MqttError {
                error: Mqtt5ReturnCodes::MqttPacketInvalidSize,
            }));
        }
    } else {
        error!("[Serializer:MqttFactory] Invalid Connect size");
        return Err(Box::new(MqttError {
            error: Mqtt5ReturnCodes::MqttPacketInvalidSize,
        }));
    }

    let connect_flag = ConnectFlag::new_by_hex(header.data[9]);
    if connect_flag.is_err() {
        error!("[Serializer:MqttFactory] Invalid Connect Flag");
        return Err(Box::new(MqttError {
            error: Mqtt5ReturnCodes::MqttRcProtocolError,
        }));
    }
    let mut connect_payload: Vec<u8> = vec![];
    for i in 10..(header.data[1] + 2) as usize {
        connect_payload.push(header.data[i]);
    }
    let payload = PayloadConnect::new_by_hex(connect_payload, connect_flag.clone().ok().unwrap());
    if payload.is_err() {
        error!("[Serializer:MqttFactory] Invalid Connect Payload");
        return Err(Box::new(MqttError {
            error: Mqtt5ReturnCodes::MqttRcProtocolError,
        }));
    }
    let payload_con = payload.ok().unwrap();
    if header.data[1] as i32 - payload_con.get_data().len() as i32 - 8 != 0 {
        error!("[Serializer:Mqtt Factory] Invalid payload size");
        return Err(Box::new(MqttError {
            error: Mqtt5ReturnCodes::MqttPacketInvalidSize,
        }));
    }
    Connect::new(connect_flag.ok().unwrap(), payload_con)
}

pub(crate) fn new_connack(header: MqttHeader) -> Result<Connack, Mqtt5ReturnCodes> {
    if header.data.len() != 4 {
        error!("[Serializer:Mqtt Factory] Invalid connack size");
        return Err(Mqtt5ReturnCodes::MqttRcProtocolError);
    }
    let connect_ackn_flag = match header.data[2] {
        0x00 => ConnectAcknowledgeFlags::Sp0,
        0x01 => ConnectAcknowledgeFlags::Sp1,
        _ => ConnectAcknowledgeFlags::Sp0,
    };
    let ret = Connack::new(
        connect_ackn_flag,
        ConnectReturnCodes::new_by_hex(header.data[3]),
    );
    Ok(ret)
}

pub(crate) fn new_publish(header: MqttHeader) -> Result<Publish, Box<dyn Error>> {
    if header.data.len() > 2 {
        if (header.data[1] + 2) as usize != header.data.len() {
            error!("[Serializer:Mqtt Factory] Invalid publish size");
            return Err(MqttError {
                error: Mqtt5ReturnCodes::MqttPacketInvalidSize,
            }
            .into());
        }
    } else {
        error!("[Serializer:Mqtt Factory] Invalid publish size");
        return Err(MqttError {
            error: Mqtt5ReturnCodes::MqttPacketInvalidSize,
        }
        .into());
    }
    let publish_flag = PublishFlag::new_by_hex(header.data[0] & 15);
    if publish_flag.is_err() {
        error!("[Serializer:Mqtt Factory] Invalid publish flag");
        return Err(MqttError {
            error: publish_flag.err().unwrap(),
        }
        .into());
    }
    let mut topic_data: Vec<u8> = vec![];
    let mut payload: Vec<u8> = vec![];
    let llsb = header.data[3];
    for i in 2..llsb as usize + 4 {
        topic_data.push(header.data[i]);
    }
    if publish_flag.clone().unwrap().get_qos() == 1 {
        let pmsb_pos = (header.data[3] as usize) + 6;
        for i in pmsb_pos..header.data.len() {
            payload.push(header.data[i]);
        }
    } else {
        let pmsb_pos = (header.data[3] as usize) + 4;
        for i in pmsb_pos..header.data.len() {
            payload.push(header.data[i]);
        }
    }

    let topic = TopicFilter::new_by_hex(topic_data[0], topic_data[1], topic_data, None)?;
    let payload_string = String::from_utf8(payload);
    if payload_string.is_err() {
        error!("[Serializer:Mqtt Factory] Invalid publish payload");
        return Err(payload_string.err().unwrap().into());
    }
    let ret = Publish::new(
        publish_flag.ok().unwrap(),
        topic,
        payload_string.ok().unwrap(),
    );
    if ret.is_err() {
        return Err(MqttError {
            error: ret.err().unwrap(),
        }
        .into());
    }
    Ok(ret.ok().unwrap())
}

pub(crate) fn new_subscribe(header: MqttHeader) -> Result<Subscribe, Box<dyn Error>> {
    let remaining_size = header.data[1] as usize;
    let mut filters = topic_filters_with_qos(&header, remaining_size)?;
    let ret = Subscribe::new(&mut filters)?;
    Ok(ret)
}

pub(crate) fn new_suback(header: MqttHeader) -> Result<Suback, Mqtt5ReturnCodes> {
    let mut suback_return_codes: Vec<SubackReturnCode> = Vec::new();
    let mut i = 4;
    while i < header.data.len() {
        match header.data[i] {
            0x80 => suback_return_codes.push(SubackReturnCode::Failure),
            0x0 => suback_return_codes.push(SubackReturnCode::MaxQoS0),
            0x1 => suback_return_codes.push(SubackReturnCode::MaxQoS1),
            0x2 => suback_return_codes.push(SubackReturnCode::MaxQoS2),
            _ => {
                error!("[Serializer:Mqtt Factory] Invalid suback QOS");
                return Err(Mqtt5ReturnCodes::MqttRcProtocolError);
            }
        }
        i += 3;
    }
    let ret = Suback::new(suback_return_codes);
    Ok(ret)
}

pub(crate) fn new_unsubscribe(header: MqttHeader) -> Result<Unsubscribe, Box<dyn Error>> {
    let remaining_size = header.data[1] as usize;
    let mut filters = topic_filters(&header, remaining_size)?;
    let ret = Unsubscribe::new(&mut filters)?;
    Ok(ret)
}

pub(crate) fn new_unsuback() -> Unsuback {
    Unsuback::new()
}

pub(crate) fn new_pingresp() -> Pingresp {
    Pingresp::new()
}
pub(crate) fn new_pingreq() -> Pingreq {
    Pingreq::new()
}

pub(crate) fn new_disconnect() -> Disconnect {
    Disconnect::new()
}

fn topic_filters_with_qos(
    header: &MqttHeader,
    remaining_size: usize,
) -> Result<Vec<TopicFilter>, Box<dyn Error>> {
    let mut j = 4;
    let mut filters: Vec<TopicFilter> = Vec::new();
    let mut size = 0;
    while j < remaining_size {
        let mut filter: Vec<u8> = Vec::new();
        let llsb = header.data[j + 1] as usize;
        let mut qos: u8 = 0;
        for k in 0..(llsb + 3) {
            if k == llsb + 2 {
                qos = header.data[j + k];
            } else {
                filter.push(header.data[j + k]);
            }
            size += 1;
        }
        let filter = TopicFilter::new_by_hex(filter[0], filter[1], filter, Option::from(qos))?;
        filters.push(filter);
        j += llsb + 3;
    }
    if size + 2 != remaining_size {
        error!("[Serializer:Mqtt Factory] Invalid topic filter size");
        return Err(Box::new(MqttError {
            error: Mqtt5ReturnCodes::MqttPacketInvalidSize,
        }));
    }
    Ok(filters)
}

fn topic_filters(
    header: &MqttHeader,
    remaining_size: usize,
) -> Result<Vec<TopicFilter>, Box<dyn Error>> {
    let mut j = 4;
    let mut filters: Vec<TopicFilter> = Vec::new();
    let mut size = 0;
    while j < remaining_size {
        let mut filter: Vec<u8> = Vec::new();
        let llsb = header.data[j + 1] as usize;
        for k in 0..(llsb + 2) {
            filter.push(header.data[j + k]);
            size += 1;
        }
        let filter = TopicFilter::new_by_hex(filter[0], filter[1], filter, None)?;
        filters.push(filter);
        j += llsb + 2;
    }
    if size + 2 != remaining_size {
        error!("[Serializer:Mqtt Factory] Invalid topic filter size");
        return Err(Box::new(MqttError {
            error: Mqtt5ReturnCodes::MqttPacketInvalidSize,
        }));
    }
    Ok(filters)
}
