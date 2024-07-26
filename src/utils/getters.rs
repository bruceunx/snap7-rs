use chrono::{DateTime, NaiveDate, Utc};
use std::convert::TryInto;
use std::time::Duration;

pub fn get_bool(bytearray: &[u8], byte_index: usize, bool_index: usize) -> bool {
    let index_value = 1 << bool_index;
    let byte_value = bytearray[byte_index];
    let current_value = byte_value & index_value;
    current_value == index_value
}

pub fn get_byte(bytearray: &[u8], byte_index: usize) -> u8 {
    bytearray[byte_index]
}

pub fn get_word(bytearray: &[u8], byte_index: usize) -> u16 {
    let data: [u8; 2] = bytearray[byte_index..byte_index + 2].try_into().unwrap();
    u16::from_be_bytes(data)
}

pub fn get_int(bytearray: &[u8], byte_index: usize) -> i16 {
    let data: [u8; 2] = bytearray[byte_index..byte_index + 2].try_into().unwrap();
    i16::from_be_bytes(data)
}

pub fn get_uint(bytearray: &[u8], byte_index: usize) -> u16 {
    get_word(bytearray, byte_index)
}

pub fn get_real(bytearray: &[u8], byte_index: usize) -> f32 {
    let data: [u8; 4] = bytearray[byte_index..byte_index + 4].try_into().unwrap();
    f32::from_bits(u32::from_be_bytes(data))
}

pub fn get_fstring(
    bytearray: &[u8],
    byte_index: usize,
    max_length: usize,
    remove_padding: bool,
) -> String {
    let data = &bytearray[byte_index..byte_index + max_length];
    let string = String::from_utf8(data.to_vec()).unwrap();

    if remove_padding {
        string.trim_end().to_string()
    } else {
        string
    }
}

pub fn get_string(bytearray: &[u8], byte_index: usize) -> String {
    let max_string_size = bytearray[byte_index] as usize;
    let str_length = bytearray[byte_index + 1] as usize;

    if str_length > max_string_size || max_string_size > 254 {
        panic!("String length error");
    }

    let data = &bytearray[byte_index + 2..byte_index + 2 + str_length];
    String::from_utf8(data.to_vec()).unwrap()
}

pub fn get_dword(bytearray: &[u8], byte_index: usize) -> u32 {
    let data: [u8; 4] = bytearray[byte_index..byte_index + 4].try_into().unwrap();
    u32::from_be_bytes(data)
}

pub fn get_dint(bytearray: &[u8], byte_index: usize) -> i32 {
    let data: [u8; 4] = bytearray[byte_index..byte_index + 4].try_into().unwrap();
    i32::from_be_bytes(data)
}

pub fn get_udint(bytearray: &[u8], byte_index: usize) -> u32 {
    get_dword(bytearray, byte_index)
}

pub fn get_s5time(bytearray: &[u8], byte_index: usize) -> String {
    let data_bytearray = &bytearray[byte_index..byte_index + 2];
    let s5time_data_int_like = format!("{:02X}{:02X}", data_bytearray[0], data_bytearray[1]);

    let time_base = match &s5time_data_int_like[0..1] {
        "0" => 10,
        "1" => 100,
        "2" => 1000,
        "3" => 10000,
        _ => panic!("This value should not be greater than 3"),
    };

    let mut s5time_bcd: i32 = 0;

    for (i, digit) in s5time_data_int_like.chars().enumerate() {
        if i > 0 {
            s5time_bcd *= 10;
            s5time_bcd += digit.to_digit(10).unwrap() as i32;
        }
    }

    let s5time_microseconds = time_base * s5time_bcd;
    let s5time = Duration::from_micros(s5time_microseconds as u64 * 1000);
    format!("{:?}", s5time)
}

pub fn get_dt(bytearray: &[u8], byte_index: usize) -> String {
    get_date_time_object(bytearray, byte_index).to_string()
}

pub fn get_date_time_object(bytearray: &[u8], byte_index: usize) -> DateTime<Utc> {
    fn bcd_to_byte(byte: u8) -> u8 {
        (byte >> 4) * 10 + (byte & 0xF)
    }
    let year = bcd_to_byte(bytearray[byte_index]) as i32;
    let year = if year < 90 { 2000 + year } else { 1900 + year };
    let month = bcd_to_byte(bytearray[byte_index + 1]);
    let day = bcd_to_byte(bytearray[byte_index + 2]);
    let hour = bcd_to_byte(bytearray[byte_index + 3]);
    let min = bcd_to_byte(bytearray[byte_index + 4]);
    let sec = bcd_to_byte(bytearray[byte_index + 5]);
    let microsec = (bcd_to_byte(bytearray[byte_index + 6]) as u32 * 10
        + (bytearray[byte_index + 7] >> 4) as u32)
        * 1000;

    NaiveDate::from_ymd_opt(year, month.into(), day.into())
        .expect("failed to parse date")
        .and_hms_micro_opt(hour.into(), min.into(), sec.into(), microsec)
        .expect("failed to parse time")
        .and_utc()
}

pub fn get_time(bytearray: &[u8], byte_index: usize) -> String {
    let data_bytearray = &bytearray[byte_index..byte_index + 4];
    let mut val = i32::from_be_bytes(data_bytearray.try_into().unwrap());

    let sign_str = if val < 0 {
        val = -val;
        "-"
    } else {
        ""
    };

    let milli_seconds = val % 1000;
    let seconds = (val / 1000) % 60;
    let minutes = (val / (1000 * 60)) % 60;
    let hours = (val / (1000 * 60 * 60)) % 24;
    let days = val / (1000 * 60 * 60 * 24);

    format!(
        "{}{}:{}:{}:{}.{}",
        sign_str,
        days,
        hours % 24,
        minutes % 60,
        seconds % 60,
        milli_seconds
    )
}

pub fn get_usint(bytearray: &[u8], byte_index: usize) -> u8 {
    bytearray[byte_index]
}

pub fn get_sint(bytearray: &[u8], byte_index: usize) -> i8 {
    bytearray[byte_index] as i8
}

pub fn get_lint(bytearray: &[u8], byte_index: usize) -> i64 {
    let data: [u8; 8] = bytearray[byte_index..byte_index + 8].try_into().unwrap();
    i64::from_be_bytes(data)
}

pub fn get_lreal(bytearray: &[u8], byte_index: usize) -> f64 {
    let data: [u8; 8] = bytearray[byte_index..byte_index + 8].try_into().unwrap();
    f64::from_bits(u64::from_be_bytes(data))
}

pub fn get_lword(bytearray: &[u8], byte_index: usize) -> u64 {
    let data: [u8; 8] = bytearray[byte_index..byte_index + 8].try_into().unwrap();
    u64::from_be_bytes(data)
}

pub fn get_ulint(bytearray: &[u8], byte_index: usize) -> u64 {
    get_lword(bytearray, byte_index)
}

pub fn get_tod(bytearray: &[u8], byte_index: usize) -> Duration {
    let len_bytearray = bytearray.len();
    let byte_range = byte_index + 4;
    if len_bytearray < byte_range {
        panic!("Date can't be extracted from bytearray. bytearray_[Index:Index+16] would cause overflow.");
    }
    let time_val = Duration::from_millis(u32::from_be_bytes(
        bytearray[byte_index..byte_range].try_into().unwrap(),
    ) as u64);
    if time_val.as_secs() >= 86400 {
        panic!(
            "Time_Of_Date can't be extracted from bytearray. Bytearray contains unexpected values."
        );
    }
    time_val
}

pub fn get_date(bytearray: &[u8], byte_index: usize) -> chrono::NaiveDate {
    use chrono::NaiveDate;

    let len_bytearray = bytearray.len();
    let byte_range = byte_index + 2;
    if len_bytearray < byte_range {
        panic!("Date can't be extracted from bytearray. bytearray_[Index:Index+16] would cause overflow.");
    }
    let date_val = NaiveDate::from_ymd_opt(1990, 1, 1).expect("failed to parse date.")
        + chrono::Duration::days(u16::from_be_bytes(
            bytearray[byte_index..byte_range].try_into().unwrap(),
        ) as i64);
    if date_val > NaiveDate::from_ymd_opt(2168, 12, 31).expect("failed to parse date.") {
        panic!("date_val is higher than specification allows.");
    }
    date_val
}

#[cfg(test)]
mod getters_tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_get_bool() {
        let bytearray = [0b10101010];
        assert!(get_bool(&bytearray, 0, 1));
        assert!(!get_bool(&bytearray, 0, 0));
    }

    #[test]
    fn test_get_byte() {
        let bytearray = [0x12];
        assert_eq!(get_byte(&bytearray, 0), 0x12);
    }

    #[test]
    fn test_get_word() {
        let bytearray = [0x12, 0x34];
        assert_eq!(get_word(&bytearray, 0), 0x1234);
    }

    #[test]
    fn test_get_int() {
        let bytearray = [0xFF, 0xD6];
        assert_eq!(get_int(&bytearray, 0), -42);
    }

    #[test]
    fn test_get_uint() {
        let bytearray = [0x12, 0x34];
        assert_eq!(get_uint(&bytearray, 0), 0x1234);
    }

    #[test]
    fn test_get_real() {
        let bytearray = [0x41, 0x20, 0x00, 0x00];
        assert_eq!(get_real(&bytearray, 0), 10.0);
    }

    #[test]
    fn test_get_fstring() {
        let bytearray = b"hello";
        assert_eq!(get_fstring(bytearray, 0, 5, true), "hello");
    }

    #[test]
    fn test_get_string() {
        let bytearray = [5, 4, b'h', b'e', b'l', b'l', b'o'];
        assert_eq!(get_string(&bytearray, 0), "hell");
    }

    #[test]
    fn test_get_dword() {
        let bytearray = [0x12, 0x34, 0x56, 0x78];
        assert_eq!(get_dword(&bytearray, 0), 0x12345678);
    }

    #[test]
    fn test_get_dint() {
        let bytearray = [0xFF, 0xFF, 0xFF, 0xC6];
        assert_eq!(get_dint(&bytearray, 0), -58);
    }

    #[test]
    fn test_get_udint() {
        let bytearray = [0x12, 0x34, 0x56, 0x78];
        assert_eq!(get_udint(&bytearray, 0), 0x12345678);
    }

    #[test]
    fn test_get_s5time() {
        let bytearray = [0x12, 0x34];
        assert_eq!(get_s5time(&bytearray, 0), "23.4s");
    }

    #[test]
    fn test_get_dt() {
        let bytearray = [0x24, 0x12, 0x12, 0x12, 0x30, 0x30, 0x30, 0x00];
        assert_eq!(get_dt(&bytearray, 0), "2024-12-12 12:30:30.300 UTC");
    }

    #[test]

    fn test_get_time() {
        let bytearray = [0x7f, 0xff, 0xff, 0xff];
        assert_eq!(get_time(&bytearray, 0), "24:20:31:23.647");
    }

    #[test]
    fn test_get_usint() {
        let bytearray = [0x12];
        assert_eq!(get_usint(&bytearray, 0), 0x12);
    }

    #[test]
    fn test_get_sint() {
        let bytearray = [0xF6];
        assert_eq!(get_sint(&bytearray, 0), -10);
    }

    #[test]
    fn test_get_lint() {
        let bytearray = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xC6];
        assert_eq!(get_lint(&bytearray, 0), -58);
    }

    #[test]
    fn test_get_lreal() {
        let bytearray = [0x40, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(get_lreal(&bytearray, 0), 10.0);
    }

    #[test]
    fn test_get_lword() {
        let bytearray = [0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF];
        assert_eq!(get_lword(&bytearray, 0), 0x1234567890ABCDEF);
    }

    #[test]
    fn test_get_ulint() {
        let bytearray = [0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF];
        assert_eq!(get_ulint(&bytearray, 0), 0x1234567890ABCDEF);
    }

    #[test]
    fn test_get_tod() {
        let bytearray = [0x00, 0x01, 0x51, 0x80];
        assert_eq!(get_tod(&bytearray, 0), Duration::from_millis(86400));
    }

    #[test]
    fn test_get_date() {
        let days_since_1990 = (NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
            - NaiveDate::from_ymd_opt(1990, 1, 1).unwrap())
        .num_days() as u16;
        let bytearray = days_since_1990.to_be_bytes();
        assert_eq!(
            get_date(&bytearray, 0),
            NaiveDate::from_ymd_opt(2024, 1, 1).expect("failed to parse date")
        );
    }
}
