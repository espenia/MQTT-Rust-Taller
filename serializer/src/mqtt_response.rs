use std::fmt::{Debug, Display, Formatter};
use std::{error::Error, fmt};

pub struct MqttError {
    pub error: Mqtt5ReturnCodes,
}

impl Display for MqttError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error type:{0}", self.error.clone() as u8)
    }
}

impl Debug for MqttError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error type:{0}", self.error.clone() as u8)
    }
}

impl Error for MqttError {}

#[derive(Clone, Debug)]
pub enum Mqtt5ReturnCodes {
    MqttRcSuccessNormalOrDisconnectionOrGrantedQos0 = 0, /* CONNACK, PUBACK, PUBREC, PUBREL, PUBCOMP, UNSUBACK, AUTH */
    /* DISCONNECT */ /* SUBACK */
    MqttRcGrantedQos1 = 1,             /* SUBACK */
    MqttRcGrantedQos2 = 2,             /* SUBACK */
    MqttRcDisconnectWithWillMsg = 4,   /* DISCONNECT */
    MqttRcNoMatchingSubscribers = 16,  /* PUBACK, PUBREC */
    MqttRcNoSubscriptionExisted = 17,  /* UNSUBACK */
    MqttRcContinueAuthentication = 24, /* AUTH */
    MqttRcReauthenticate = 25,         /* AUTH */

    MqttRcUnspecified = 128, /* CONNACK, PUBACK, PUBREC, SUBACK, UNSUBACK, DISCONNECT */
    MqttRcMalformedPacket = 129, /* CONNACK, DISCONNECT */
    MqttRcProtocolError = 130, /* DISCONNECT */
    MqttRcImplementationSpecific = 131, /* CONNACK, PUBACK, PUBREC, SUBACK, UNSUBACK, DISCONNECT */
    MqttRcUnsupportedProtocolVersion = 132, /* CONNACK */
    MqttRcClientidNotValid = 133, /* CONNACK */
    MqttRcBadUsernameOrPassword = 134, /* CONNACK */
    MqttRcNotAuthorized = 135, /* CONNACK, PUBACK, PUBREC, SUBACK, UNSUBACK, DISCONNECT */
    MqttRcServerUnavailable = 136, /* CONNACK */
    MqttRcServerBusy = 137,  /* CONNACK, DISCONNECT */
    MqttRcBanned = 138,      /* CONNACK */
    MqttRcServerShuttingDown = 139, /* DISCONNECT */
    MqttRcBadAuthenticationMethod = 140, /* CONNACK */
    MqttRcKeepAliveTimeout = 141, /* DISCONNECT */
    MqttRcSessionTakenOver = 142, /* DISCONNECT */
    MqttRcTopicFilterInvalid = 143, /* SUBACK, UNSUBACK, DISCONNECT */
    MqttRcTopicNameInvalid = 144, /* CONNACK, PUBACK, PUBREC, DISCONNECT */
    MqttRcPacketIdInUse = 145, /* PUBACK, SUBACK, UNSUBACK */
    MqttRcPacketIdNotFound = 146, /* PUBREL, PUBCOMP */
    MqttRcReceiveMaximumExceeded = 147, /* DISCONNECT */
    MqttRcTopicAliasInvalid = 148, /* DISCONNECT */
    MqttPacketInvalidSize = 149, //TOO_LARGE = 149, /* CONNACK, PUBACK, PUBREC, DISCONNECT */
    MqttRcMessageRateTooHigh = 150, /* DISCONNECT */
    MqttRcQuotaExceeded = 151, /* PUBACK, PUBREC, SUBACK, DISCONNECT */
    MqttRcAdministrativeAction = 152, /* DISCONNECT */
    MqttRcPayloadFormatInvalid = 153, /* CONNACK, DISCONNECT */
    MqttRcRetainNotSupported = 154, /* CONNACK, DISCONNECT */
    MqttRcQosNotSupported = 155, /* CONNACK, DISCONNECT */
    MqttRcUseAnotherServer = 156, /* CONNACK, DISCONNECT */
    MqttRcServerMoved = 157, /* CONNACK, DISCONNECT */
    MqttRcSharedSubsNotSupported = 158, /* SUBACK, DISCONNECT */
    MqttRcConnectionRateExceeded = 159, /* CONNACK, DISCONNECT */
    MqttRcMaximumConnectTime = 160, /* DISCONNECT */
    MqttRcSubscriptionIdsNotSupported = 161, /* SUBACK, DISCONNECT */
    MqttRcWildcardSubsNotSupported = 162, /* SUBACK, DISCONNECT */
}
