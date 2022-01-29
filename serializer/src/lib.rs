pub mod constants_and_structs;
pub mod mqtt_factory;
pub mod mqtt_response;
pub mod packets;
pub mod tools;

pub use crate::constants_and_structs::connect_flag::ConnectFlag;
pub use crate::constants_and_structs::payload_connect::PayloadConnect;
pub use crate::packets::connect::Connect;
use std::error::Error;
use std::option::Option::None;

pub use crate::constants_and_structs::connect_return_codes::ConnectReturnCodes;
pub use crate::constants_and_structs::mqtt_constants::ConnectAcknowledgeFlags;
pub use crate::constants_and_structs::mqtt_constants::ConnectReturnCode;
pub use crate::constants_and_structs::mqtt_constants::PacketType;
pub use crate::constants_and_structs::mqtt_constants::SubackReturnCode;
pub use crate::constants_and_structs::publish_flag::PublishFlag;
pub use crate::constants_and_structs::topic_filter::TopicFilter;
pub use crate::mqtt_factory::MqttHeader;
pub use crate::mqtt_response::Mqtt5ReturnCodes;
pub use crate::packets::connack::Connack;
pub use crate::packets::disconnect::Disconnect;
pub use crate::packets::pingreq::Pingreq;
pub use crate::packets::pingresp::Pingresp;
pub use crate::packets::puback::Puback;
pub use crate::packets::publish::Publish;
pub use crate::packets::suback::Suback;
pub use crate::packets::subscribe::Subscribe;
pub use crate::packets::unsuback::Unsuback;
pub use crate::packets::unsubscribe::Unsubscribe;

pub fn new_mqtt_header(data: Vec<u8>) -> Result<MqttHeader, Mqtt5ReturnCodes> {
    MqttHeader::new(data)
}

pub fn new_connect_flag(
    clean_session: Option<bool>,
    will_flag: Option<bool>,
    will_qosb1: Option<bool>,
    will_qosb2: Option<bool>,
    will_retain: Option<bool>,
    password_flag: Option<bool>,
    username_flag: Option<bool>,
) -> Result<ConnectFlag, Mqtt5ReturnCodes> {
    ConnectFlag::new(
        clean_session,
        will_flag,
        will_qosb1,
        will_qosb2,
        will_retain,
        password_flag,
        username_flag,
    )
}

pub fn new_payload_connect(
    client_identifier: String,
    will_topic: String,
    will_message: String,
    username: String,
    password: String,
    keep_alive: u16,
) -> Result<PayloadConnect, Mqtt5ReturnCodes> {
    PayloadConnect::new(
        client_identifier,
        will_topic,
        will_message,
        username,
        password,
        keep_alive,
    )
}

pub fn new_connect(
    connect_flag: ConnectFlag,
    payload_connect: PayloadConnect,
) -> Result<Connect, Box<dyn Error>> {
    Connect::new(connect_flag, payload_connect)
}

pub fn new_connect_by_hex(data: MqttHeader) -> Result<Connect, Box<dyn Error>> {
    mqtt_factory::new_connect(data)
}

pub fn new_connect_return_code(reject_reason: ConnectReturnCode) -> ConnectReturnCodes {
    ConnectReturnCodes::new(reject_reason)
}

pub fn new_connack(
    connect_acknowledge_flags: ConnectAcknowledgeFlags,
    connect_return_code: ConnectReturnCodes,
) -> Connack {
    Connack::new(connect_acknowledge_flags, connect_return_code)
}

pub fn new_connack_by_hex(data: MqttHeader) -> Result<Connack, Mqtt5ReturnCodes> {
    mqtt_factory::new_connack(data)
}

pub fn new_publish_packet_flags(
    retain: Option<bool>,
    qosb1: Option<bool>,
    qosb2: Option<bool>,
    dup_flag: Option<bool>,
) -> Result<PublishFlag, Mqtt5ReturnCodes> {
    PublishFlag::new(retain, qosb1, qosb2, dup_flag)
}

pub fn new_topic_filter(topic: String) -> Result<TopicFilter, Mqtt5ReturnCodes> {
    let mut data = vec![0, topic.len() as u8];
    data.append(&mut topic.clone().into_bytes());
    TopicFilter::new(0, (topic.len()) as u8, data, None, topic)
}

pub fn new_topic_filter_by_hex(data: Vec<u8>) -> Result<TopicFilter, Box<dyn Error>> {
    TopicFilter::new_by_hex(0, (data.len() - 2) as u8, data, None)
}

pub fn new_topic_filter_with_qos_by_hex(
    data: Vec<u8>,
    qos: u8,
) -> Result<TopicFilter, Box<dyn Error>> {
    TopicFilter::new_by_hex(0, (data.len() - 2) as u8, data, Option::from(qos))
}

pub fn new_topic_filter_with_qos(topic: String, qos: u8) -> Result<TopicFilter, Mqtt5ReturnCodes> {
    let mut data = vec![0, topic.len() as u8];
    data.append(&mut topic.clone().into_bytes());
    TopicFilter::new(0, (topic.len()) as u8, data, Option::from(qos), topic)
}

pub fn new_publish(
    publish_packet_flags: PublishFlag,
    publish_topic: TopicFilter,
    payload: String,
) -> Result<Publish, Mqtt5ReturnCodes> {
    Publish::new(publish_packet_flags, publish_topic, payload)
}

pub fn new_puback() -> Puback {
    Puback::new()
}

pub fn new_publish_by_hex(data: MqttHeader) -> Result<Publish, Box<dyn Error>> {
    mqtt_factory::new_publish(data)
}

pub fn new_subscribe(mut topic_filters: Vec<TopicFilter>) -> Result<Subscribe, Box<dyn Error>> {
    Subscribe::new(&mut topic_filters)
}

pub fn new_subscribe_by_hex(data: MqttHeader) -> Result<Subscribe, Box<dyn Error>> {
    mqtt_factory::new_subscribe(data)
}

pub fn new_suback(suback_ret_codes: Vec<SubackReturnCode>) -> Suback {
    Suback::new(suback_ret_codes)
}
pub fn new_suback_by_hex(data: MqttHeader) -> Result<Suback, Mqtt5ReturnCodes> {
    mqtt_factory::new_suback(data)
}

pub fn new_unsubscribe(mut topic_filters: Vec<TopicFilter>) -> Result<Unsubscribe, Box<dyn Error>> {
    Unsubscribe::new(&mut topic_filters)
}
pub fn new_unsubscribe_by_hex(data: MqttHeader) -> Result<Unsubscribe, Box<dyn Error>> {
    mqtt_factory::new_unsubscribe(data)
}

pub fn new_unsuback() -> Unsuback {
    Unsuback::new()
}
pub fn new_unsuback_by_hex() -> Unsuback {
    mqtt_factory::new_unsuback()
}

pub fn new_pingresp_by_hex() -> Pingresp {
    mqtt_factory::new_pingresp()
}

pub fn new_pingreq_by_hex() -> Pingreq {
    mqtt_factory::new_pingreq()
}

pub fn new_pingresp() -> Pingresp {
    Pingresp::new()
}

pub fn new_pingreq() -> Pingreq {
    Pingreq::new()
}

pub fn new_disconnect() -> Disconnect {
    Disconnect::new()
}

pub fn new_disconnect_by_hex() -> Disconnect {
    mqtt_factory::new_disconnect()
}

#[cfg(test)]
mod tests {
    use crate::constants_and_structs::connect_flag::ConnectFlag;
    use crate::constants_and_structs::connect_return_codes::ConnectReturnCodes;
    use crate::constants_and_structs::mqtt_constants::{
        ConnectAcknowledgeFlags, ConnectReturnCode, PacketType, SubackReturnCode,
        LENGTH_LSB_CONNECT, LENGTH_MSB_CONNECT, PACKET_FLAGS_CONNACK, PACKET_FLAGS_CONNECT,
        PROTOCOL_NAME_M, PROTOCOL_NAME_Q, PROTOCOL_NAME_T, PROTOCOL_VERSION,
        REMAINING_LENGTH_CONNACK, SUBACK_PACKET_FLAGS, SUBSCRIBE_PACKET_FLAGS,
        UNSUBSCRIBE_PACKET_FLAGS,
    };
    use crate::constants_and_structs::payload_connect::PayloadConnect;
    use crate::constants_and_structs::publish_flag::PublishFlag;
    use crate::constants_and_structs::topic_filter::TopicFilter;
    use crate::mqtt_factory;
    use crate::mqtt_factory::MqttHeader;
    use crate::packets::connack::Connack;
    use crate::packets::connect::Connect;
    use crate::packets::disconnect::Disconnect;
    use crate::packets::pingreq::Pingreq;
    use crate::packets::pingresp::Pingresp;
    use crate::packets::puback::Puback;
    use crate::packets::publish::Publish;
    use crate::packets::suback::Suback;
    use crate::packets::subscribe::Subscribe;
    use crate::packets::unsuback::Unsuback;
    use crate::packets::unsubscribe::Unsubscribe;
    use std::option::Option::None;
    use std::panic::panic_any;

    #[test]
    fn create_connect_flag() {
        assert_eq!(
            ConnectFlag::new(
                None,
                None,
                Option::from(true),
                None,
                Option::from(true),
                None,
                None
            )
            .ok()
            .unwrap()
            .hex_value(),
            0x28
        );
    }

    #[test]
    fn create_connect_flag_invalid_qos() {
        assert!(ConnectFlag::new(
            None,
            None,
            Option::from(true),
            Option::from(true),
            Option::from(true),
            None,
            None
        )
        .is_err());
    }
    #[test]
    fn create_connect_flag_invalid_qos_hex() {
        assert!(ConnectFlag::new_by_hex(0x1C).is_err());
    }

    #[test]
    fn create_connect_payload() {
        let connect_flag = ConnectFlag::new(
            Option::from(true),
            Option::from(true),
            None,
            None,
            None,
            Option::from(true),
            Option::from(true),
        );
        let payload_connect = PayloadConnect::new(
            "42".to_string(),
            "a topicñ".to_string(),
            "".to_string(),
            "user".to_string(),
            "pass".to_string(),
            60,
        );
        if payload_connect.is_err() {
            panic_any(payload_connect.err())
        }
        let payload_struct = payload_connect.ok().unwrap().clone();
        let payload = payload_struct.get_data().clone();
        let payload_size = payload.len().clone();
        let connect_flag_struct = connect_flag.ok().unwrap().clone();
        let connect_flag_hex = connect_flag_struct.hex_value().clone();
        let mut vector = vec![
            ((PacketType::CONNECT as u8) << 4) | PACKET_FLAGS_CONNECT,
            8 + payload_size as u8,
            LENGTH_LSB_CONNECT,
            LENGTH_MSB_CONNECT,
            PROTOCOL_NAME_M,
            PROTOCOL_NAME_Q,
            PROTOCOL_NAME_T,
            PROTOCOL_NAME_T,
            PROTOCOL_VERSION,
            connect_flag_hex.clone(),
        ];
        vector.append(&mut payload.clone());
        let connect = Connect::new(connect_flag_struct.clone(), payload_struct);
        assert_eq!(connect.ok().unwrap().get_data(), vector.clone())
    }

    #[test]
    fn create_connect_payload_hex() {
        let connect_flag = ConnectFlag::new(
            Option::from(true),
            Option::from(true),
            None,
            None,
            None,
            Option::from(true),
            Option::from(true),
        );
        let payload_connect = PayloadConnect::new(
            "42".to_string(),
            "a topic".to_string(),
            "mess".to_string(),
            "user".to_string(),
            "pass".to_string(),
            60,
        );
        if payload_connect.is_err() {
            panic_any(payload_connect.err())
        }
        let payload_struct = payload_connect.ok().unwrap().clone();
        let payload = payload_struct.get_data().clone();
        let payload_size = payload.len().clone();
        let connect_flag_struct = connect_flag.ok().unwrap().clone();
        let connect_flag_hex = connect_flag_struct.hex_value().clone();
        let mut vector = vec![
            ((PacketType::CONNECT as u8) << 4) | PACKET_FLAGS_CONNECT,
            8 + payload_size as u8,
            LENGTH_LSB_CONNECT,
            LENGTH_MSB_CONNECT,
            PROTOCOL_NAME_M,
            PROTOCOL_NAME_Q,
            PROTOCOL_NAME_T,
            PROTOCOL_NAME_T,
            PROTOCOL_VERSION,
            connect_flag_hex.clone(),
        ];
        vector.append(&mut payload.clone());
        let connect = mqtt_factory::new_connect(MqttHeader::new(vector.clone()).ok().unwrap());
        assert_eq!(connect.ok().unwrap().get_data(), vector.clone())
    }

    #[test]
    fn create_connack() {
        let connect_return_code = ConnectReturnCodes::new(ConnectReturnCode::ConnectionAccepted);
        let connack = Connack::new(ConnectAcknowledgeFlags::Sp0, connect_return_code);
        assert_eq!(
            connack.get_data(),
            vec![
                ((PacketType::CONNACK as u8) << 4) | PACKET_FLAGS_CONNACK,
                2,
                ConnectAcknowledgeFlags::Sp0 as u8,
                connect_return_code.hex_value()
            ]
        )
    }

    #[test]
    fn is_connect() {
        let header = MqttHeader::new(vec![0x10, 0x0]).ok().unwrap();
        assert_eq!(
            header.get_control_packet_type() as u8,
            PacketType::CONNECT as u8
        )
    }

    #[test]
    fn is_connack() {
        let header = MqttHeader::new(vec![0x20, 0x0]).ok().unwrap();
        assert_eq!(
            header.get_control_packet_type() as u8,
            PacketType::CONNACK as u8
        )
    }

    #[test]
    fn create_new_connect() {
        let connect_flag = ConnectFlag::new(Option::from(true), None, None, None, None, None, None);
        let connect_flag_struct = connect_flag.ok().unwrap().clone();
        let connect_flag_hex = connect_flag_struct.hex_value().clone();
        let payload = PayloadConnect::new(
            "42".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            60,
        );
        if payload.is_err() {
            panic_any(payload.err())
        }
        let mut data = vec![
            ((PacketType::CONNECT as u8) << 4) | PACKET_FLAGS_CONNECT,
            8 + payload.clone().ok().unwrap().get_data().len() as u8,
            LENGTH_LSB_CONNECT,
            LENGTH_MSB_CONNECT,
            PROTOCOL_NAME_M,
            PROTOCOL_NAME_Q,
            PROTOCOL_NAME_T,
            PROTOCOL_NAME_T,
            PROTOCOL_VERSION,
            connect_flag_hex.clone(),
        ];
        data.append(&mut payload.clone().ok().unwrap().get_data().clone());
        let header = MqttHeader::new(data);
        let connect = mqtt_factory::new_connect(header.ok().unwrap())
            .ok()
            .unwrap();

        let valid_connect =
            Connect::new(connect_flag_struct.clone(), payload.ok().unwrap().clone());

        assert_eq!(valid_connect.ok().unwrap().get_data(), connect.get_data())
    }

    #[test]
    fn create_new_connack() {
        let header = MqttHeader::new(vec![
            (PacketType::CONNACK as u8) << 4 | PACKET_FLAGS_CONNACK,
            REMAINING_LENGTH_CONNACK,
            0x0,
            0x0,
        ]);
        let connack = mqtt_factory::new_connack(header.ok().unwrap())
            .ok()
            .unwrap();

        let connect_return_code = ConnectReturnCodes::new(ConnectReturnCode::ConnectionAccepted);
        let valid_connack = Connack::new(ConnectAcknowledgeFlags::Sp0, connect_return_code);
        assert_eq!(valid_connack.get_data(), connack.get_data());
    }

    #[test]
    fn create_new_publish() {
        let flag = PublishFlag::new(None, Option::from(true), None, None)
            .ok()
            .unwrap();
        let filter = TopicFilter::new(
            0,
            4,
            vec![0, 4, 'a' as u8, '/' as u8, 'b' as u8, 'y' as u8],
            None,
            "a/by".to_string(),
        )
        .ok()
        .unwrap();
        let valid_publish = Publish::new(flag.clone(), filter.clone(), "payloadasd".to_string())
            .ok()
            .unwrap();
        let mut data = vec![
            (PacketType::PUBLISH as u8) << 4 | flag.clone().hex_value(),
            18,
        ];
        data.append(&mut filter.get_filter().clone());
        data.push(0);
        data.push(0);
        data.append(&mut "payloadasd".to_string().into_bytes());
        let header = MqttHeader::new(data).ok().unwrap();
        let publish = mqtt_factory::new_publish(header).ok().unwrap();
        assert_eq!(publish.get_data(), valid_publish.get_data())
    }

    #[test]
    fn create_new_puback() {
        let valid_puback = Puback::new();
        let puback = Puback::new();
        assert_eq!(puback.get_data(), valid_puback.get_data())
    }

    #[test]
    fn create_new_unsubscribe() {
        let filter = TopicFilter::new(
            0,
            4,
            vec![0, 4, 'a' as u8, '/' as u8, 'b' as u8, 'y' as u8],
            None,
            "a/by".to_string(),
        )
        .ok()
        .unwrap();
        let mut filters = vec![];
        filters.push(filter.clone());
        filters.push(filter.clone());
        filters.push(filter.clone());
        filters.push(filter.clone());
        let remaining_length: u8 = (4 + 2) * 4 + 2;
        let valid_unsubscribe = Unsubscribe::new(&mut filters).ok().unwrap();
        let mut data = vec![
            (PacketType::UNSUSCRIBE as u8) << 4 | UNSUBSCRIBE_PACKET_FLAGS,
            remaining_length,
            0,
            0,
        ];
        let mut tp_filters = topic_filters(filters);
        data.append(&mut tp_filters);
        let header = MqttHeader::new(data).ok().unwrap();
        let unsubscribe = mqtt_factory::new_unsubscribe(header).ok().unwrap();
        assert_eq!(unsubscribe.get_data(), valid_unsubscribe.get_data())
    }

    fn topic_filters_with_qos(filters: Vec<TopicFilter>) -> Vec<u8> {
        let mut ret: Vec<u8> = vec![];
        for filter in filters {
            let mut f: Vec<u8> = filter.get_filter().clone();
            ret.append(&mut f);
            ret.push(filter.get_qos())
        }
        ret
    }

    fn topic_filters(filters: Vec<TopicFilter>) -> Vec<u8> {
        let mut ret: Vec<u8> = vec![];
        for filter in filters {
            let mut f: Vec<u8> = filter.get_filter().clone();
            ret.append(&mut f);
        }
        ret
    }

    #[test]
    fn create_new_subscribe() {
        let filter = TopicFilter::new(
            0,
            4,
            vec![0, 4, 'a' as u8, '/' as u8, 'b' as u8, 'y' as u8],
            Option::from(0 as u8),
            "a/by".to_string(),
        )
        .ok()
        .unwrap();
        let mut filters = vec![];
        filters.push(filter.clone());
        filters.push(filter.clone());
        filters.push(filter.clone());
        filters.push(filter.clone());
        let remaining_length: u8 = (4 + 3) * 4 + 2;
        let valid_subscribe = Subscribe::new(&mut filters).ok().unwrap();
        let mut data = vec![
            (PacketType::SUBSCRIBE as u8) << 4 | SUBSCRIBE_PACKET_FLAGS,
            remaining_length,
            0,
            0,
        ];
        let mut tp_filters = topic_filters_with_qos(filters);
        data.append(&mut tp_filters);
        let header = MqttHeader::new(data).ok().unwrap();
        let subscribe = mqtt_factory::new_subscribe(header).ok().unwrap();
        assert_eq!(subscribe.get_data(), valid_subscribe.get_data())
    }

    #[test]
    fn create_new_suback() {
        let data = vec![
            (PacketType::SUBACK as u8) << 4 | SUBACK_PACKET_FLAGS,
            3 * 3,
            0,
            1,
            SubackReturnCode::MaxQoS0 as u8,
            0,
            1,
            SubackReturnCode::MaxQoS1 as u8,
            0,
            1,
            SubackReturnCode::Failure as u8,
        ];
        let valid_suback = Suback::new(vec![
            SubackReturnCode::MaxQoS0,
            SubackReturnCode::MaxQoS1,
            SubackReturnCode::Failure,
        ]);
        let header = MqttHeader::new(data).ok().unwrap();
        let suback = mqtt_factory::new_suback(header).ok().unwrap();
        assert_eq!(suback.get_data(), valid_suback.get_data())
    }

    #[test]
    fn create_new_unsuback() {
        let valid_unsuback = Unsuback::new();
        let unsuback = mqtt_factory::new_unsuback();
        assert_eq!(unsuback.get_data(), valid_unsuback.get_data())
    }

    #[test]
    fn create_new_pingresp() {
        let valid_pingresp = Pingresp::new();
        let pingresp = mqtt_factory::new_pingresp();
        assert_eq!(pingresp.get_data(), valid_pingresp.get_data())
    }

    #[test]
    fn create_new_pingreq() {
        let valid_pingreq = Pingreq::new();
        let pingreq = mqtt_factory::new_pingreq();
        assert_eq!(pingreq.get_data(), valid_pingreq.get_data())
    }

    #[test]
    fn create_new_disconnect() {
        let valid_disconnect = Disconnect::new();
        let disconnect = mqtt_factory::new_disconnect();
        assert_eq!(disconnect.get_data(), valid_disconnect.get_data())
    }
}
