// 8..0: Flags
// 92..8: Hand
// 126..92: Pool

pub const ANY_TON: u8 = 0b00000001;
pub const ANY_NAN: u8 = 0b00000010;
pub const ANY_SHA: u8 = 0b00000100;
pub const ANY_PEI: u8 = 0b00001000;

pub const MASK_GROUP_ANY: u8 = 0b00001111;

pub const NOT_ANY_TON: u8 = 0b00000000;
pub const NOT_ANY_NAN: u8 = 0b00010000;
pub const NOT_ANY_SHA: u8 = 0b00100000;
pub const NOT_ANY_PEI: u8 = 0b00110000;

pub const MASK_GROUP_NOT_ANY: u8 = 0b00110000;

pub const MASK_FALSE_IF_RON: u8 = 0b01000000;

pub const MASK_TRUE_ALWAYS: u8 = 0b10000000;

pub fn set_hand(info: &mut u128, hand: [u8; 14]) {
    let mut shift = 8;
    for i in 0..14 {
        *info |= ((hand[i] as u128) << shift);
        shift += 6;
    }
}

pub fn get_hand(info: &u128) -> [u8; 14] {
    let mut hand = [0; 14];
    let mut shift = 8;
    for i in 0..14 {
        hand[i] = ((info >> shift) & 0b111111) as u8;
        shift += 6;
    }
    hand
}

pub fn set_pool(info: &mut u128, pool: [bool; 34]) {
    for i in 0..34 {
        if pool[i] {
            *info |= 1 << (i + 92);
        }
    }
}

pub fn get_pool(info: &u128) -> [bool; 34] {
    let mut pool = [false; 34];
    for i in 0..34 {
        pool[i] = info & (1 << (i + 92)) != 0;
    }
    pool
}
