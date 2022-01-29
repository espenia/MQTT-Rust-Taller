use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone)]
pub struct UserQos {
    user: String,
    qos: u8,
}

impl Serialize for UserQos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("UserQos", 2)?;
        state.serialize_field("user", &self.user)?;
        state.serialize_field("qos", &self.qos)?;
        state.end()
    }
}

impl PartialEq for UserQos {
    fn eq(&self, other: &Self) -> bool {
        other.qos == self.qos && self.user == other.user
    }
}

const FIELDS: &'static [&'static str] = &["user", "qos"];
impl<'de> Deserialize<'de> for UserQos {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            User,
            Qos,
        }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`user` or `qos`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "user" => Ok(Field::User),
                            "qos" => Ok(Field::Qos),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct UserQosVisitor;

        impl<'de> Visitor<'de> for UserQosVisitor {
            type Value = UserQos;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct UserQos")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<UserQos, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let user = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let qos = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(UserQos::new(user, qos))
            }

            fn visit_map<V>(self, mut map: V) -> Result<UserQos, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut user = None;
                let mut qos = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::User => {
                            if user.is_some() {
                                return Err(de::Error::duplicate_field("user"));
                            }
                            user = Some(map.next_value()?);
                        }
                        Field::Qos => {
                            if qos.is_some() {
                                return Err(de::Error::duplicate_field("qos"));
                            }
                            qos = Some(map.next_value()?);
                        }
                    }
                }
                let user = user.ok_or_else(|| de::Error::missing_field("user"))?;
                let qos = qos.ok_or_else(|| de::Error::missing_field("qos"))?;
                Ok(UserQos::new(user, qos))
            }
        }

        deserializer.deserialize_struct("UserQos", FIELDS, UserQosVisitor)
    }
}

impl UserQos {
    pub fn new(user: String, qos: u8) -> Self {
        UserQos { user, qos }
    }

    pub fn get_user(&self) -> String {
        self.user.clone()
    }

    pub fn get_qos(&self) -> u8 {
        self.qos
    }
}
