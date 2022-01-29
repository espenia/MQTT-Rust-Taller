use crate::json_helper;
use crate::json_helper::write_topic_subs;
use crate::packets::user_qos::UserQos;
use serializer::{new_suback, new_topic_filter_with_qos, SubackReturnCode, Subscribe, TopicFilter};
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::Write;
use std::net::TcpStream;
use tracing::{error, warn};

/// Logica de paquete Subscribe
pub fn resolve_subscribe(
    stream: &mut TcpStream,
    subscribe: Subscribe,
    user: (u32, String),
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut topics = subscribe.get_topics();
    let mut topic_subs = json_helper::read_topic_subs()?;

    for t in topics.clone() {
        if t.get_topic().contains('*') {
            topics.append(&mut wild_card_topics(
                t.get_topic(),
                topic_subs.clone(),
                t.get_qos(),
            ));
        }
    }
    topics = remove_duplicates_and_wild_cards(topics);

    let mut suback_payload: Vec<serializer::SubackReturnCode> = vec![];
    let mut new_sub = vec![];
    for topic in topics {
        let topic_str = topic.get_topic();
        if topic_subs.contains_key(&topic_str.clone()) {
            let mut userqos = topic_subs[&topic_str].clone();
            if !contains_user(userqos.clone(), (*user.1).to_string()) {
                new_sub.push(topic_str.clone());
                userqos.push(UserQos::new((*user.1).to_string(), topic.get_qos()));
            } else {
                userqos = replace_qos(userqos, (*user.1).to_string(), topic.get_qos());
            }
            topic_subs.insert(topic_str, userqos);
            suback_payload.push(suback_ret_code(topic.get_qos()));
        } else {
            warn!("[Server:Subscribe] No existe el topic {:?}", topic_str);
            suback_payload.push(SubackReturnCode::Failure);
            //si no existe tirar suback failure
        }
    }
    if send_suback(stream, suback_payload).is_err() {
        error!("[Server:Subscribe] Error al mandar suback")
    }
    let result = write_topic_subs(topic_subs)?;
    if result {
        Ok(new_sub.clone())
    } else {
        Ok(vec![])
    }
}

fn send_suback(
    stream: &mut TcpStream,
    suback_payload: Vec<serializer::SubackReturnCode>,
) -> io::Result<usize> {
    let suback = new_suback(suback_payload);
    stream.write(&suback.get_data())
}

fn suback_ret_code(qos: u8) -> SubackReturnCode {
    return match qos {
        0 => SubackReturnCode::MaxQoS0,
        1 => SubackReturnCode::MaxQoS1,
        _ => {
            error!("[Server:Subscribe] topic {:?} QoS invalido", qos);
            SubackReturnCode::Failure
        }
    };
}

fn contains_user(userqos: Vec<UserQos>, user: String) -> bool {
    for user_qos in userqos {
        if user_qos.get_user() == user {
            return true;
        }
    }
    false
}

fn replace_qos(mut userqos: Vec<UserQos>, user: String, qos: u8) -> Vec<UserQos> {
    for i in 0..userqos.len() {
        if userqos[i].get_user() == user.clone() {
            userqos[i] = UserQos::new(user, qos);
            break;
        }
    }
    userqos
}
///Uso de WildCards para multiples subscribe de topics
pub fn wild_card_topics(
    topic: String,
    topic_subs: HashMap<String, Vec<UserQos>>,
    qos: u8,
) -> Vec<TopicFilter> {
    let mut topics: Vec<TopicFilter> = vec![];
    for key in topic_subs.keys() {
        let wild_card = WildCard::new(&topic);
        if wild_card.matches(key) {
            match new_topic_filter_with_qos(key.clone(), qos) {
                Ok(t) => topics.push(t),
                Err(_) => {
                    error!("error al crear topic wild card")
                }
            }
        }
    }
    topics
}

pub fn remove_duplicates_and_wild_cards(topics: Vec<TopicFilter>) -> Vec<TopicFilter> {
    let mut topic_unique = vec![];
    for topic in &topics {
        if !topic.get_topic().contains('*') {
            if topic_unique.clone().is_empty() {
                topic_unique.push(topic.clone());
            }
            for un_topic in topic_unique.clone().into_iter() {
                if un_topic.get_topic() != topic.get_topic() {
                    topic_unique.push(topic.clone());
                    break;
                }
            }
        }
    }
    topic_unique
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WildCard {
    pattern: Vec<State>,
}

#[derive(Debug, Clone, PartialEq)]
struct State {
    next_char: Option<char>,
    has_wildcard: bool,
}

impl WildCard {
    /// Creo un WildCard, con un pattern que debe contener un *
    /// ejemplo: cat*_dog -> cat_perro_dog, cat1_dog, etc
    pub fn new(pattern: &str) -> WildCard {
        let mut simplified: Vec<State> = Vec::with_capacity(pattern.len());
        let mut prev_was_star = false;
        for current_char in pattern.chars() {
            match current_char {
                '*' => {
                    prev_was_star = true;
                }
                _ => {
                    let s = State {
                        next_char: Some(current_char),
                        has_wildcard: prev_was_star,
                    };
                    simplified.push(s);
                    prev_was_star = false;
                }
            }
        }

        if !pattern.is_empty() {
            let final_state = State {
                next_char: None,
                has_wildcard: prev_was_star,
            };
            simplified.push(final_state);
        }

        WildCard {
            pattern: simplified,
        }
    }

    ///Verifico si mi pattern, cumple con el string que se le envia.
    pub fn matches(&self, input: &str) -> bool {
        if self.pattern.is_empty() {
            return input.is_empty();
        }
        let mut pattern_idx = 0;
        for input_char in input.chars() {
            match self.pattern.get(pattern_idx) {
                None => {
                    return false;
                }
                Some(p) if p.next_char == Some('?') || p.next_char == Some(input_char) => {
                    pattern_idx += 1;
                }
                Some(p) if p.has_wildcard => {
                    if p.next_char == None {
                        return true;
                    }
                }
                _ => {
                    if pattern_idx == 0 {
                        return false;
                    };
                    pattern_idx -= 1;
                    while let Some(pattern) = self.pattern.get(pattern_idx) {
                        if pattern.has_wildcard {
                            if pattern.next_char == Some('?')
                                || pattern.next_char == Some(input_char)
                            {
                                pattern_idx += 1;
                            }
                            break;
                        }
                        if pattern_idx == 0 {
                            return false;
                        };
                        pattern_idx -= 1;
                    }
                }
            }
        }
        self.pattern[pattern_idx].next_char.is_none()
    }
}
