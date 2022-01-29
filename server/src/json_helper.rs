//! Modulo de procesamiento de archivos
use crate::packets::user_qos::UserQos;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
/// `json_helper` es una coleccion de funciones que se encargan de la escritura en archivos
/// json que se utilizan para guardar la infomación del servidor como retain messages, queue messages,
/// subscribers, users, etc.

pub mod json {}

pub fn read_retain_messages() -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
    let data = read_from_path("./retain_messages.json".to_string());
    let retain_message: HashMap<String, Vec<String>> = serde_json::from_str(&data)?;
    Ok(retain_message)
}

pub fn read_q_messages() -> Result<HashMap<String, Vec<Vec<u8>>>, Box<dyn Error>> {
    let data = read_from_path("./queue_messages.json".to_string());
    let q_messages: HashMap<String, Vec<Vec<u8>>> = serde_json::from_str(&data)?;
    Ok(q_messages)
}

pub fn write_q_messages(data: HashMap<String, Vec<Vec<u8>>>) -> Result<bool, Box<dyn Error>> {
    let current_data = read_q_messages()?;
    let new_data = append_maps_q_mess(current_data, data);
    write_to_path("./queue_messages.json".to_string(), new_data)
}

pub fn write_remove_q_messages(
    data: HashMap<String, Vec<Vec<u8>>>,
    user: String,
) -> Result<bool, Box<dyn Error>> {
    let current_data = read_q_messages()?;
    let new_data = append_maps_remove_q_mess(current_data, data, user);
    write_to_path("./queue_messages.json".to_string(), new_data)
}

pub fn write_retain_messages(data: HashMap<String, Vec<String>>) -> Result<bool, Box<dyn Error>> {
    let current_data = read_retain_messages()?;
    let new_data = append_maps_ret(current_data, data);
    write_to_path("./retain_messages.json".to_string(), new_data)
}

pub fn read_topic_subs() -> Result<HashMap<String, Vec<UserQos>>, Box<dyn Error>> {
    let data = read_from_path("./topic_subscribers.json".to_string());
    let topic_subs: HashMap<String, Vec<UserQos>> = serde_json::from_str(&data)?;
    Ok(topic_subs)
}

pub fn write_topic_subs(data: HashMap<String, Vec<UserQos>>) -> Result<bool, Box<dyn Error>> {
    let current_data = read_topic_subs()?;
    let new_data = append_maps_sub(current_data, data);
    write_to_path("./topic_subscribers.json".to_string(), new_data)
}

pub fn write_topic_unsubs(
    data: HashMap<String, Vec<UserQos>>,
    user: String,
    topics: Vec<String>,
) -> Result<bool, Box<dyn Error>> {
    let current_data = read_topic_subs()?;
    let new_data = append_maps_unsub(current_data, data, user, topics);
    write_to_path("./topic_subscribers.json".to_string(), new_data)
}

pub fn read_users() -> Result<HashMap<u32, String>, Box<dyn Error>> {
    let data = read_from_path("./users.json".to_string());
    let users: HashMap<u32, String> = serde_json::from_str(&data)?;
    Ok(users)
}

pub fn write_users(data: HashMap<u32, String>) -> Result<bool, Box<dyn Error>> {
    let current_data = read_users()?;
    let new_data = append_maps_users(current_data, data);
    write_to_path("./users.json".to_string(), new_data)
}

pub fn read_user_db() -> Result<HashMap<String, String>, Box<dyn Error>> {
    let data = read_from_path("./user_db.json".to_string());
    let users: HashMap<String, String> = serde_json::from_str(&data)?;
    Ok(users)
}

fn write_to_path<T: serde::ser::Serialize + std::cmp::Eq + std::hash::Hash, U: serde::Serialize>(
    path: String,
    data: HashMap<T, U>,
) -> Result<bool, Box<dyn Error>> {
    let mut timeout_count = 100;
    let message = serde_json::to_string(&data).unwrap();
    loop {
        let result = fs::write(path.clone(), message.as_bytes());
        if result.is_ok() {
            break;
        } else {
            if result
                .as_ref()
                .err()
                .unwrap()
                .to_string()
                .contains("No such file or directory")
            {
                return Err(Box::new(result.err().unwrap()));
            }
            timeout_count -= 1;
            if timeout_count <= 0 {
                return Err(Box::new(result.err().unwrap()));
            }
        }
    }
    Ok(true)
}

fn read_from_path(path: String) -> String {
    let mut data = String::new();
    loop {
        let str = fs::read_to_string(&path);
        match str {
            Ok(string) => {
                data = string;
                break;
            }
            Err(err) => {
                if err.to_string().contains("No such file or directory") {
                    return data;
                }
            }
        }
    }
    data
}

fn append_maps_remove_q_mess(
    mut old_map: HashMap<String, Vec<Vec<u8>>>,
    new_map: HashMap<String, Vec<Vec<u8>>>,
    user: String,
) -> HashMap<String, Vec<Vec<u8>>> {
    for k in old_map.clone().keys() {
        if k.clone() == user.clone() {
            old_map.remove(&user.clone());
            old_map.insert(user.clone(), vec![]);
        } else {
            let mut values = new_map[k].clone();
            values.append(&mut old_map[k].clone());
            let mut unique: Vec<Vec<u8>> = vec![];
            for value in values {
                if !unique.contains(&value) {
                    unique.push(value);
                }
            }
            old_map.insert(k.clone(), unique);
        }
    }
    old_map
}

fn append_maps_q_mess(
    mut old_map: HashMap<String, Vec<Vec<u8>>>,
    new_map: HashMap<String, Vec<Vec<u8>>>,
) -> HashMap<String, Vec<Vec<u8>>> {
    for k in old_map.clone().keys() {
        let mut values = new_map[k].clone();
        values.append(&mut old_map[k].clone());
        let mut unique: Vec<Vec<u8>> = vec![];
        for value in values {
            if !unique.contains(&value) {
                unique.push(value);
            }
        }
        old_map.insert(k.clone(), unique);
    }

    for k in new_map.keys() {
        old_map
            .entry(k.clone())
            .or_insert_with(|| new_map[k].clone());
    }

    old_map
}

fn append_maps_users(
    mut old_map: HashMap<u32, String>,
    new_map: HashMap<u32, String>,
) -> HashMap<u32, String> {
    for i in new_map.keys() {
        for j in old_map.clone().keys() {
            if old_map[j] == new_map[i] && i != j {
                old_map.remove(j);
            }
        }
        if !old_map.contains_key(i) {
            old_map.insert(*i, new_map[i].to_string());
        }
    }
    old_map
}

fn append_maps_ret(
    mut old_map: HashMap<String, Vec<String>>,
    new_map: HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    for k in old_map.clone().keys() {
        let mut values = new_map[k].clone();
        values.append(&mut old_map[k].clone());
        let mut unique: Vec<String> = vec![];
        for value in values {
            if !unique.contains(&value) {
                unique.push(value);
            }
        }
        old_map.insert(k.clone(), unique);
    }

    for k in new_map.keys() {
        old_map
            .entry(k.clone())
            .or_insert_with(|| new_map[k].clone());
    }

    old_map
}

fn append_maps_sub(
    mut old_map: HashMap<String, Vec<UserQos>>,
    new_map: HashMap<String, Vec<UserQos>>,
) -> HashMap<String, Vec<UserQos>> {
    for k in old_map.clone().keys() {
        if new_map.contains_key(&k.clone()) {
            let mut values2 = new_map[k].clone();
            values2.append(&mut old_map[k].clone());
            let mut unique: Vec<UserQos> = vec![];
            let mut users: Vec<String> = vec![];
            for value in values2 {
                if !users.contains(&value.get_user()) {
                    unique.push(value.clone());
                    users.push(value.get_user());
                }
            }
            old_map.insert(k.clone(), unique);
        }
    }
    for k in new_map.keys() {
        old_map
            .entry(k.clone())
            .or_insert_with(|| new_map[k].clone());
    }
    old_map
}

fn append_maps_unsub(
    mut old_map: HashMap<String, Vec<UserQos>>,
    new_map: HashMap<String, Vec<UserQos>>,
    user: String,
    topics: Vec<String>,
) -> HashMap<String, Vec<UserQos>> {
    for k in old_map.clone().keys() {
        if new_map.contains_key(&k.clone()) {
            let mut values = old_map[k].clone();
            values.append(&mut new_map[k].clone());
            let mut unique: Vec<UserQos> = vec![];
            let mut users: Vec<String> = vec![];
            if topics.contains(k) {
                users.push(user.clone())
            }
            for value in values {
                if !users.contains(&value.get_user()) {
                    unique.push(value.clone());
                    users.push(value.get_user());
                }
            }
            old_map.insert(k.clone(), unique);
        }
    }
    old_map
}
