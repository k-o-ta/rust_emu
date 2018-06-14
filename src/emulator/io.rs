use std::io;
pub trait Io {
    fn io_in8(address: u16) -> u8 {
        match address {
            0x03f8 => {
                let mut input = String::new();
                let ret: char;
                match io::stdin().read_line(&mut input) {
                    Ok(_) => ret = input.remove(0),
                    Err(e) => panic!("Cant't read line : {}", e),
                };
                ret as u8
            }
            _ => 0,
        }
    }

    fn io_out8(address: u16, value: u8) {
        match address {
            0x03f8 => {
                print!("{}", value as char);
            }
            _ => {}
        }
    }
}
