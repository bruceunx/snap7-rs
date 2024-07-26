use regex;
use std::time::Duration;

pub fn set_bool(
    bytearray: &mut [u8],
    byte_index: usize,
    bool_index: usize,
    value: bool,
) -> Result<(), String> {
    if bool_index > 7 {
        return Err(format!("bool_index {} out of range", bool_index));
    }

    let mask = 1 << bool_index;
    if value {
        bytearray[byte_index] |= mask;
    } else {
        bytearray[byte_index] &= !mask;
    }
    Ok(())
}

pub fn set_byte(bytearray: &mut [u8], byte_index: usize, value: u8) {
    bytearray[byte_index] = value;
}

pub fn set_word(bytearray: &mut [u8], byte_index: usize, value: u16) {
    bytearray[byte_index..byte_index + 2].copy_from_slice(&value.to_be_bytes());
}

pub fn set_int(bytearray: &mut [u8], byte_index: usize, value: i16) {
    bytearray[byte_index..byte_index + 2].copy_from_slice(&value.to_be_bytes());
}

pub fn set_uint(bytearray: &mut [u8], byte_index: usize, value: u16) {
    bytearray[byte_index..byte_index + 2].copy_from_slice(&value.to_be_bytes());
}

pub fn set_real(bytearray: &mut [u8], byte_index: usize, value: f32) {
    bytearray[byte_index..byte_index + 4].copy_from_slice(&value.to_be_bytes());
}

pub fn set_dword(bytearray: &mut [u8], byte_index: usize, value: u32) {
    bytearray[byte_index..byte_index + 4].copy_from_slice(&value.to_be_bytes());
}

pub fn set_dint(bytearray: &mut [u8], byte_index: usize, value: i32) {
    bytearray[byte_index..byte_index + 4].copy_from_slice(&value.to_be_bytes());
}

pub fn set_udint(bytearray: &mut [u8], byte_index: usize, value: u32) {
    bytearray[byte_index..byte_index + 4].copy_from_slice(&value.to_be_bytes());
}

pub fn set_time(bytearray: &mut [u8], byte_index: usize, time_string: &str) -> Result<(), String> {
    let duration = parse_time_string(time_string)?;
    let millis = duration.as_millis() as i32;
    bytearray[byte_index..byte_index + 4].copy_from_slice(&millis.to_be_bytes());
    Ok(())
}

pub fn parse_time_string(time_string: &str) -> Result<Duration, String> {
    let re = regex::Regex::new(r"(-?)(\d+):(\d+):(\d+):(\d+).(\d+)").unwrap();
    if let Some(caps) = re.captures(time_string) {
        let sign = if &caps[1] == "-" { -1 } else { 1 };
        let days: u64 = caps[2].parse().unwrap();
        let hours: u64 = caps[3].parse().unwrap();
        let minutes: u64 = caps[4].parse().unwrap();
        let seconds: u64 = caps[5].parse().unwrap();
        let millis: u64 = caps[6].parse().unwrap();
        let total_millis = (((((days * 24 + hours) * 60 + minutes) * 60 + seconds) * 1000 + millis)
            as i64
            * sign) as u64;
        Ok(Duration::from_millis(total_millis))
    } else {
        Err(format!("Invalid time string: {}", time_string))
    }
}

pub fn set_usint(bytearray: &mut [u8], byte_index: usize, value: u8) {
    bytearray[byte_index] = value;
}

pub fn set_sint(bytearray: &mut [u8], byte_index: usize, value: i8) {
    bytearray[byte_index] = value as u8;
}

pub fn set_lreal(bytearray: &mut [u8], byte_index: usize, value: f64) {
    bytearray[byte_index..byte_index + 8].copy_from_slice(&value.to_be_bytes());
}

pub fn set_char(bytearray: &mut [u8], byte_index: usize, value: char) -> Result<(), String> {
    if value.is_ascii() {
        bytearray[byte_index] = value as u8;
        Ok(())
    } else {
        Err(format!("Non-ASCII character: {}", value))
    }
}

pub fn set_date(
    bytearray: &mut [u8],
    byte_index: usize,
    value: chrono::NaiveDate,
) -> Result<(), String> {
    let base_date = chrono::NaiveDate::from_ymd_opt(1990, 1, 1).expect("failed to get base date");
    if value < base_date
        || value > chrono::NaiveDate::from_ymd_opt(2168, 12, 31).expect("failed to get base date")
    {
        return Err(format!("Date out of range: {}", value));
    }
    let days = (value - base_date).num_days() as i16;
    bytearray[byte_index..byte_index + 2].copy_from_slice(&days.to_be_bytes());
    Ok(())
}

#[cfg(test)]
mod setters_tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_set_bool() {
        let mut data = vec![0; 1];
        set_bool(&mut data, 0, 0, true).unwrap();
        assert_eq!(data, vec![1]);
        set_bool(&mut data, 0, 0, false).unwrap();
        assert_eq!(data, vec![0]);
    }

    #[test]
    fn test_set_byte() {
        let mut data = vec![0; 1];
        set_byte(&mut data, 0, 255);
        assert_eq!(data, vec![255]);
    }

    #[test]
    fn test_set_word() {
        let mut data = vec![0; 2];
        set_word(&mut data, 0, 65535);
        assert_eq!(data, vec![255, 255]);
    }

    #[test]
    fn test_set_int() {
        let mut data = vec![0; 2];
        set_int(&mut data, 0, -32768);
        assert_eq!(data, vec![128, 0]);
    }

    #[test]
    fn test_set_date() {
        let mut data = vec![0; 2];
        let date = NaiveDate::from_ymd_opt(2024, 3, 27).unwrap();
        set_date(&mut data, 0, date).unwrap();
        assert_eq!(data, vec![48, 216]);
    }
    #[test]
    fn test_set_uint() {
        let mut bytearray = [0u8; 10];
        set_uint(&mut bytearray, 2, 0x1234);
        assert_eq!(bytearray[2], 0x12);
        assert_eq!(bytearray[3], 0x34);
    }

    #[test]
    fn test_set_real() {
        let mut bytearray = [0u8; 10];
        set_real(&mut bytearray, 2, 12.34);
        assert_eq!(bytearray[2..6], 12.34f32.to_be_bytes());
    }

    #[test]
    fn test_set_dword() {
        let mut bytearray = [0u8; 10];
        set_dword(&mut bytearray, 2, 0x12345678);
        assert_eq!(bytearray[2], 0x12);
        assert_eq!(bytearray[3], 0x34);
        assert_eq!(bytearray[4], 0x56);
        assert_eq!(bytearray[5], 0x78);
    }

    #[test]
    fn test_set_dint() {
        let mut bytearray = [0u8; 10];
        set_dint(&mut bytearray, 2, -12345678);
        assert_eq!(bytearray[2..6], (-12345678i32).to_be_bytes());
    }

    #[test]
    fn test_set_udint() {
        let mut bytearray = [0u8; 10];
        set_udint(&mut bytearray, 2, 0x12345678);
        assert_eq!(bytearray[2], 0x12);
        assert_eq!(bytearray[3], 0x34);
        assert_eq!(bytearray[4], 0x56);
        assert_eq!(bytearray[5], 0x78);
    }

    #[test]
    fn test_set_time() {
        let mut bytearray = [0u8; 10];
        set_time(&mut bytearray, 2, "0:0:0:1:0.0").unwrap();
        assert_eq!(bytearray[2..6], 1000i32.to_be_bytes());
    }

    #[test]
    fn test_set_usint() {
        let mut bytearray = [0u8; 10];
        set_usint(&mut bytearray, 2, 0x12);
        assert_eq!(bytearray[2], 0x12);
    }

    #[test]
    fn test_set_sint() {
        let mut bytearray = [0u8; 10];
        set_sint(&mut bytearray, 2, -5);
        assert_eq!(bytearray[2], (-5i8) as u8);
    }

    #[test]
    fn test_set_lreal() {
        let mut bytearray = [0u8; 10];
        set_lreal(&mut bytearray, 2, 12.34);
        assert_eq!(bytearray[2..10], 12.34f64.to_be_bytes());
    }

    #[test]
    fn test_set_char() {
        let mut bytearray = [0u8; 10];
        set_char(&mut bytearray, 2, 'A').unwrap();
        assert_eq!(bytearray[2], b'A');
    }

    #[test]
    fn test_set_char_non_ascii() {
        let mut bytearray = [0u8; 10];
        let result = set_char(&mut bytearray, 2, 'ç');
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_time_string_valid() {
        let duration = parse_time_string("0:0:0:1:0.0").unwrap();
        assert_eq!(duration.as_millis(), 1000);
    }

    #[test]
    fn test_parse_time_string_invalid() {
        let result = parse_time_string("invalid time");
        assert!(result.is_err());
    }
}
