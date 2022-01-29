use crate::json_helper;
use crate::json_helper::write_topic_unsubs;
use crate::packets::subscribe::{remove_duplicates_and_wild_cards, WildCard};
use crate::packets::user_qos::UserQos;
use serializer::{new_topic_filter, new_unsuback, TopicFilter, Unsubscribe};
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::Write;
use std::net::TcpStream;
use tracing::{error, warn};

/// Logica de paquete Unsubscribe
pub fn resolve_unsubscribe(
    stream: &mut TcpStream,
    unsubscribe: Unsubscribe,
    user: (u32, String),
) -> Result<(), Box<dyn Error>> {
    let mut topics = unsubscribe.get_topic_filters();
    let mut unsub_topic = vec![];
    let mut topic_subs = json_helper::read_topic_subs()?;

    for t in topics.clone() {
        if t.get_topic().contains('*') {
            topics.append(&mut wild_card_topics_unsub(
                t.get_topic(),
                topic_subs.clone(),
            ));
        }
    }
    topics = remove_duplicates_and_wild_cards(topics);

    for topic in topics {
        let topic_str = topic.get_topic();
        if topic_subs.contains_key(&topic_str.clone()) {
            let userqos_list = topic_subs[&topic_str].clone();
            let userqos = find_userqos(userqos_list.clone(), (*user.1).to_string());
            match userqos {
                Ok(userqos) => {
                    let updated_userqos_list = remove_from_qos(userqos_list, userqos);
                    topic_subs.remove(&topic_str);
                    unsub_topic.push(topic_str.clone());
                    topic_subs.insert(topic_str, updated_userqos_list);
                }
                Err(_) => {
                    warn!(
                        "[Server:Unsubscribe] No existe el user {:?}",
                        (*user.1).to_string()
                    );
                }
            }
        } else {
            warn!("[Server:Unsubscribe] No existe el topic {:?}", topic_str);
            //si no existe tirar suback failure
        }
    }
    if send_unsuback(stream).is_err() {
        error!("[Server:Subscribe] Error al mandar suback")
    }
    let _result = write_topic_unsubs(topic_subs, user.1, unsub_topic)?;
    Ok(())
}

fn send_unsuback(stream: &mut TcpStream) -> io::Result<usize> {
    let unsuback = new_unsuback();
    stream.write(&unsuback.get_data())
}

fn find_userqos(userqos_list: Vec<UserQos>, user: String) -> Result<UserQos, bool> {
    for user_qos in userqos_list {
        if user_qos.get_user() == user {
            return Ok(user_qos);
        }
    }
    Err(false)
}

fn remove_from_qos(userqos_list: Vec<UserQos>, userqos: UserQos) -> Vec<UserQos> {
    let index = userqos_list.iter().position(|x| *x == userqos).unwrap();
    let mut updated_userqos_list = userqos_list;
    updated_userqos_list.remove(index);
    updated_userqos_list
}

pub fn wild_card_topics_unsub(
    topic: String,
    topic_subs: HashMap<String, Vec<UserQos>>,
) -> Vec<TopicFilter> {
    let mut topics: Vec<TopicFilter> = vec![];
    for key in topic_subs.keys() {
        let wild_card = WildCard::new(&topic);
        if wild_card.matches(key) {
            match new_topic_filter(key.clone()) {
                Ok(t) => topics.push(t),
                Err(_) => {
                    error!("error al crear topic wild card")
                }
            }
        }
    }
    topics
}
