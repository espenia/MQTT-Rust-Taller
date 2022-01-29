// rust castea los u8 a decimal
pub(crate) fn to_bin8(value: u8) -> String {
    let mut bit = to_bin(value);
    match bit.len() {
        0 => bit = "00000000".to_string(),
        1 => bit = "0000000".to_string() + &*bit,
        2 => bit = "000000".to_string() + &*bit,
        3 => bit = "00000".to_string() + &*bit,
        4 => bit = "0000".to_string() + &*bit,
        5 => bit = "000".to_string() + &*bit,
        6 => bit = "00".to_string() + &*bit,
        7 => bit = "0".to_string() + &*bit,
        _ => {}
    }
    bit
}

pub(crate) fn to_bin4(value: u8) -> String {
    let mut bit = to_bin(value);
    match bit.len() {
        0 => bit = "0000".to_string(),
        1 => bit = "000".to_string() + &*bit,
        2 => bit = "00".to_string() + &*bit,
        3 => bit = "0".to_string() + &*bit,
        _ => {}
    }
    bit
}

fn to_bin(mut value: u8) -> String {
    let mut bit = String::new();
    while value != 0 {
        let remainder: u8 = (value % 2) as u8;
        if remainder == 0 {
            bit = "0".to_string() + &*bit
        } else {
            bit = "1".to_string() + &*bit
        }
        value /= 2;
    }
    bit
}

// rust castea los u8 a decimal
pub(crate) fn to_hex(mut value: i64) -> u8 {
    let mut hex: u8 = 0;
    let mut j: i64 = 1;
    while value != 0 {
        let remainder: i64 = value % 10;
        hex += (remainder * j) as u8;
        j *= 2;
        value /= 10;
    }
    hex
}
