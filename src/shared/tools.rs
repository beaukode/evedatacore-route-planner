pub fn system_id_to_u16(system_id: u32) -> Result<u16, std::num::ParseIntError> {
    let converted = match system_id {
        30000000..=30099999 => system_id % 100000,
        32000000..=32099999 => 30000 + (system_id % 100000),
        34000000..=34099999 => 40000 + (system_id % 100000),
        _ => system_id,
    };

    Ok(converted as u16)
}

pub fn u16_to_system_id(value: u16) -> u32 {
    match value {
        0..=29999 => 30000000 + value as u32,
        30000..=39999 => 32000000 + (value as u32 - 30000),
        40000..=49999 => 34000000 + (value as u32 - 40000),
        _ => value as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_id_conversion() {
        assert_eq!(system_id_to_u16(30018456).unwrap(), 18456);
        assert_eq!(system_id_to_u16(32001234).unwrap(), 31234);
        assert_eq!(system_id_to_u16(34000004).unwrap(), 40004);
    }

    #[test]
    fn test_u16_to_system_id() {
        assert_eq!(u16_to_system_id(18456), 30018456);
        assert_eq!(u16_to_system_id(31234), 32001234);
        assert_eq!(u16_to_system_id(40004), 34000004);
    }
}
