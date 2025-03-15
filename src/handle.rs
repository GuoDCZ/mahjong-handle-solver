use super::store;
use riichi::agenda::AgendaName;
use riichi::hand::{AgendaResult, PartitionedHand};

pub struct Context {
    tsumo: bool,
    east: bool,
    south: bool,
    west: bool,
    north: bool,
}

impl Context {
    pub fn check_flags(&self, flags: u8) -> bool {
        if flags & store::MASK_TRUE_ALWAYS != 0 {
            true
        } else if flags & store::MASK_FALSE_IF_RON != 0 && self.tsumo {
            false
        } else if flags
            & ((self.east as u8) << 0
                | (self.south as u8) << 1
                | (self.west as u8) << 2
                | (self.north as u8) << 3)
            != 0
        {
            true
        } else {
            match flags & store::MASK_GROUP_NOT_ANY {
                store::NOT_ANY_TON => !self.east,
                store::NOT_ANY_NAN => !self.south,
                store::NOT_ANY_SHA => !self.west,
                store::NOT_ANY_PEI => !self.north,
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Handle {
    pub hand: [u8; 14],
    pub pool: [bool; 34],
    pub flags: u8,
}

impl Handle {
    pub fn default() -> Self {
        Handle {
            hand: [0; 14],
            pool: [false; 34],
            flags: 0,
        }
    }

    pub fn to_u128(&self) -> u128 {
        let mut info: u128 = 0;
        store::set_hand(&mut info, self.hand);
        store::set_pool(&mut info, self.pool);
        info |= self.flags as u128;
        info
    }

    pub fn from_u128(info: u128) -> Self {
        Handle {
            hand: store::get_hand(&info),
            pool: store::get_pool(&info),
            flags: (info & 0b11111111) as u8,
        }
    }

    fn tile_to_string(tile: u8, last_suit: char) -> (String, char) {
        let suit = tile / 9;
        let number = tile % 9 + 1;
        let suit = match suit {
            0 => 'm',
            1 => 'p',
            2 => 's',
            3 => 'z',
            _ => unreachable!(),
        };
        if suit == last_suit || last_suit == 'x' {
            (format!("{}", number), suit)
        } else {
            (format!("{}{}", last_suit, number), suit)
        }
    }

    pub fn hand_to_string(tiles: &[u8]) -> String {
        let mut last_suit = 'x';
        let mut result = String::new();
        for tile in tiles {
            let (tile_str, suit) = Self::tile_to_string(*tile, last_suit);
            result.push_str(&tile_str);
            last_suit = suit;
        }
        result.push(last_suit);
        result
    }

    pub fn handle_to_string(handle: &Handle) -> String {
        let mut result = Self::hand_to_string(&handle.hand);
        result.push_str(" ".repeat(20 - result.len()).as_str());
        if handle.flags & store::MASK_FALSE_IF_RON != 0 {
            result.push_str(" _TSUMO__");
        } else if handle.flags & store::MASK_TRUE_ALWAYS != 0 {
            result.push_str(" _ALWAYS_");
        } else {
            result.push_str(format!(" {:08b}", handle.flags).as_str());
        }
        result
    }
}
