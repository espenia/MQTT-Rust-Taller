//http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718028

//FIXED
#[derive(Clone, Copy, Debug)]
pub enum PacketType {
    //BYTE 2/2
    CONNECT = 0x01,
    CONNACK = 0x02,
    PUBLISH = 0x03,
    PUBACK = 0x04,
    PUBREC = 0x05,
    PUBREL = 0x06,
    PUBCOMP = 0x07,
    SUBSCRIBE = 0x08,
    SUBACK = 0x09,
    UNSUSCRIBE = 0x0A,
    UNSUBACK = 0x0B,
    PINGREQ = 0x0C,
    PINGRESP = 0x0D,
    DISCONNECT = 0x0E,
}

// CONNECT
// HEADER
pub(crate) const PACKET_FLAGS_CONNECT: u8 = 0x00; // B 1/2
                                                  //B2 remaining length
pub(crate) const LENGTH_MSB_CONNECT: u8 = 0x00; // B 3
pub(crate) const LENGTH_LSB_CONNECT: u8 = 0x04; // B 4
pub(crate) const PROTOCOL_NAME_M: u8 = 0x04d; //B 5
pub(crate) const PROTOCOL_NAME_Q: u8 = 0x051; // B 6
pub(crate) const PROTOCOL_NAME_T: u8 = 0x054; // B 7,8
pub(crate) const PROTOCOL_VERSION: u8 = 0x04; // B 9
                                              // B 10 ConnectFlag
                                              // pub(crate) const KEEP_ALIVE_MSB: u8 = 0x00; // B 11
                                              // pub(crate) const KEEP_ALIVE_LSB: u8 = 0x0A; // B 12
                                              // PAYLOAD
                                              // B 13 - 36 Client Identifier (Follows 1.5.3 UTF-8 ENCODING)
                                              // B 14 Will Topic Flag dependent (FD) (Follows 1.5.3 UTF-8 ENCODING) http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#RFC3629
                                              // B 15 Will Message FD  (Follows 1.5.3 NOT UTF-8 ENCODING)
                                              // B 16 User Name FD  (Follows 1.5.3 UTF-8 ENCODING)
                                              // B 17 Password FD (Follows 1.5.3 NOT UTF-8 ENCODING)

//CONNACK
pub(crate) const PACKET_FLAGS_CONNACK: u8 = 0x00; // B 1/2
pub(crate) const REMAINING_LENGTH_CONNACK: u8 = 0x02; // B 2

#[derive(Clone, Copy, Debug)]
pub enum ConnectAcknowledgeFlags {
    // B 3
    Sp0 = 0x00,
    Sp1 = 0x01,
}

#[derive(Clone, Debug)]
pub enum ConnectReturnCode {
    // B 4
    ConnectionAccepted = 0x00,
    InvalidProtocol = 0x01,
    IdentifierRejected = 0x02,
    ServerUnavailable = 0x03,
    BadUserNameOrPassword = 0x04,
    NotAuthorized = 0x05,
    CloseConnection = 0x100, //TODO ver que valor le podemos poner.
}

//PUBLISH
// PublishFlags B 1/2
//B2 remaining length
// Payload B 10 - Remaining length

//PUBACK  - Qos1

pub(crate) const PUBACK_PACKET_FLAGS: u8 = 0x00; // b 1/2
pub(crate) const PUBACK_REMAINING_LENGTH: u8 = 0x02; // b 2
                                                     // b 3 Packet Identifier MSB
                                                     // b 4 Packet Identifier LSB

// PUBREC  - Qos2 P1

//pub(crate) const PUBREC_PACKET_FLAGS: u8 = 0x00; // b 1/2
//pub(crate) const PUBREC_REMAINING_LENGTH: u8 = 0x02; // b 2
// b 3 Packet Identifier MSB
// b 4 Packet Identifier LSB

// PUBREL  - Qos2 P2

//pub(crate) const PUBREL_PACKET_FLAGS: u8 = 0x00; // b 1/2
//pub(crate) const PUBREL_REMAINING_LENGTH: u8 = 0x02; // b 2
// b 3 Packet Identifier MSB
// b 4 Packet Identifier LSB

// PUBCOMP  - Qos2 P3

//pub(crate) const PUBCOMP_PACKET_FLAGS: u8 = 0x00; // b 1/2
//pub(crate) const PUBCOMP_REMAINING_LENGTH: u8 = 0x02; // b 2
// b 3 Packet Identifier MSB
// b 4 Packet Identifier LSB

// SUBSCRIBE  HEADER

pub(crate) const SUBSCRIBE_PACKET_FLAGS: u8 = 0x02; // b 1/2
                                                    // b 2 remaining length

// SUBSCRIBE PAYLOAD (Follows 1.5.3 NOT UTF-8 ENCODING)
// b 5 Length MSB
// b 6 Length LSB
// b 3..N Topic Filter

// SUBACK  HEADER

pub(crate) const SUBACK_PACKET_FLAGS: u8 = 0x00; // b 1/2
                                                 // b 3 Packet Identifier MSB
                                                 // b 4 Packet Identifier LSB

// SUBACK PAYLOAD

#[derive(Clone, Copy)]
pub enum SubackReturnCode {
    // b 5
    Failure = 0x80,
    MaxQoS0 = 0x00,
    MaxQoS1 = 0x01,
    MaxQoS2 = 0x02,
}

// UNSUBSCRIBE  HEADER

pub(crate) const UNSUBSCRIBE_PACKET_FLAGS: u8 = 0x02; // b 1/2
                                                      // b 2 remaining length
                                                      // b 3 Packet Identifier MSB
                                                      // b 4 Packet Identifier LSB

// UNSUBSCRIBE PAYLOAD (Follows 1.5.3 NOT UTF-8 ENCODING)
// b 5 Length MSB
// b 6 Length LSB
// b 7..N Topic Filter

// UNSUBACK

pub(crate) const UNSUBACK_PACKET_FLAGS: u8 = 0x00; // b 1/2
pub(crate) const UNSUBACK_REMAINING_LENGTH: u8 = 0x02; // b 2
                                                       // b 3 Packet Identifier MSB
                                                       // b 4 Packet Identifier LSB

// PINGREQ

pub(crate) const PINGREQ_PACKET_FLAGS: u8 = 0x00; // b 1/2
pub(crate) const PINGREQ_REMAINING_LENGTH: u8 = 0x0; // b 2

// PINGRESP

pub(crate) const PINGRESP_PACKET_FLAGS: u8 = 0x00; // b 1/2
pub(crate) const PINGRESP_REMAINING_LENGTH: u8 = 0x0; // b 2

// DISCONNECT

pub(crate) const DISCONNECT_PACKET_FLAGS: u8 = 0x00; // b 1/2
pub(crate) const DISCONNECT_REMAINING_LENGTH: u8 = 0x0; // b 2
