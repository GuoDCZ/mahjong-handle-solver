use std::vec;

use crate::store::MASK_TRUE_ALWAYS;

use super::store;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Green,
    Yellow,
    None,
}

pub type ColorResult = [Color; 14];

pub fn color_result_to_index(color_result: &ColorResult) -> u32 {
    color_result.into_iter().fold(0, |acc, color| {
        acc * 3
            + match color {
                Color::Green => 0,
                Color::Yellow => 1,
                Color::None => 2,
            }
    })
}

pub fn parse_color_result(s: &str) -> ColorResult {
    let mut result = [Color::None; 14];
    for (i, c) in s.chars().enumerate() {
        if i >= 14 {
            break;
        }
        result[i] = match c {
            'G' | 'g' => Color::Green,
            'Y' | 'y' => Color::Yellow,
            _ => Color::None,
        };
    }
    result
}

pub fn color_result_to_string(color_result: &ColorResult) -> String {
    color_result
        .iter()
        .map(|color| match color {
            Color::Green => 'G',
            Color::Yellow => 'Y',
            Color::None => 'N',
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
pub struct Context {
    tsumo: bool,
    east: bool,
    south: bool,
    west: bool,
    north: bool,
}

impl Context {
    pub fn new() -> Self {
        Context {
            tsumo: false,
            east: false,
            south: false,
            west: false,
            north: false,
        }
    }

    pub fn check_flags(&self, flags: u8) -> bool {
        if flags & store::MASK_TRUE_ALWAYS != 0 {
            true
        } else if flags & store::MASK_FALSE_IF_RON != 0 && self.tsumo {
            false
        } else if flags & store::MASK_GROUP_ANY != 0 {
            flags
                & ((self.east as u8) << 0
                    | (self.south as u8) << 1
                    | (self.west as u8) << 2
                    | (self.north as u8) << 3)
                != 0
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

    pub fn parse_context(s: &str) -> Self {
        let mut context = Context::new();
        for c in s.chars() {
            match c {
                'T' | 't' => context.tsumo = true,
                'E' | 'e' => context.east = true,
                'S' | 's' => context.south = true,
                'W' | 'w' => context.west = true,
                'N' | 'n' => context.north = true,
                _ => {}
            }
        }
        context
    }
}

pub type Hand = [u8; 14];

#[derive(Debug, Clone, Copy)]
pub struct Handle {
    pub hand: Hand,
    pub pool: [bool; 34],
    pub flags: u8,
}

impl PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        self.hand == other.hand
    }
}

impl Eq for Handle {}

impl PartialOrd for Handle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.hand.partial_cmp(&other.hand)
    }
}

impl Ord for Handle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hand.cmp(&other.hand)
    }
}

impl Handle {
    pub fn best_1st() -> Self {
        let hand = [2, 3, 4, 6, 11, 12, 13, 20, 21, 22, 23, 24, 25, 6];
        let pool = hand.iter().fold([false; 34], |mut pool, &tile| {
            pool[tile as usize] = true;
            pool
        });
        Handle {
            hand,
            pool,
            flags: store::MASK_TRUE_ALWAYS,
        }
    }

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

    pub fn from_string(s: &str) -> Self {
        let mut hand = vec![];
        let mut buf = vec![];
        for c in s.chars() {
            match c {
                '1'..='9' => buf.push(c as u8 - b'1'),
                'm' | 'p' | 's' | 'z' => {
                    let suit = match c {
                        'm' => 0,
                        'p' => 1,
                        's' => 2,
                        'z' => 3,
                        _ => unreachable!(),
                    };
                    for tile in buf.iter() {
                        hand.push(suit * 9 + tile);
                    }
                    buf.clear();
                }
                _ => {}
            }
        }
        let hand = hand.try_into().unwrap();
        let pool = [false; 34];
        let flags = MASK_TRUE_ALWAYS;
        Handle { hand, pool, flags }
    }

    pub fn match_context(&self, context: &Context) -> bool {
        context.check_flags(self.flags)
    }

    pub fn get_color_result(self, other: &Handle) -> ColorResult {
        let mut result = [Color::None; 14];
        let mut hand = self.hand;
        for i in 0..14 {
            if self.hand[i] == other.hand[i] {
                result[i] = Color::Green;
                hand[i] = u8::MAX;
            }
        }
        for i in 0..14 {
            if result[i] == Color::None {
                for j in 0..14 {
                    if hand[j] == other.hand[i] {
                        result[i] = Color::Yellow;
                        hand[j] = u8::MAX;
                        break;
                    }
                }
            }
        }
        result
    }

    pub fn match_color_result(self, other: &Handle, color_result: &ColorResult) -> bool {
        let mut hand = self.hand;
        for i in 0..14 {
            if (color_result[i] == Color::Green) != (hand[i] == other.hand[i]) {
                return false;
            }
            if color_result[i] == Color::Green {
                hand[i] = u8::MAX;
            }
        }
        // println!("Here");
        for i in 0..14 {
            let mut found = false;
            if color_result[i] == Color::Green {
                continue;
            }
            for j in 0..14 {
                if hand[j] == other.hand[i] {
                    if color_result[i] != Color::Yellow {
                        // println!("Here {} {}", i, j);
                        return false;
                    }
                    hand[j] = u8::MAX;
                    found = true;
                    break;
                }
            }
            if !found && color_result[i] == Color::Yellow {
                // println!("Here {}", i);
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_play_ground() {
        let handle = Handle::from_string("24888m678p22333s3m");
        let other = Handle::from_string("678m678p12233s55z1s");
        // let color_result = parse_color_result("ggygnnnnnnnnyy");
        println!(
            "{:?}",
            color_result_to_string(&handle.get_color_result(&other))
        );
        // assert_eq!(handle.get_color_result(&other), color_result);
    }

    #[test]
    fn test_color_result_to_index() {
        use Color::*;
        let color_result: ColorResult = [
            Green, Green, Green, Green, Green, Green, Green, Green, Green, Green, Green, Green,
            Green, Green,
        ];
        assert_eq!(color_result_to_index(&color_result), 0);

        let color_result: ColorResult = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        ];
        assert_eq!(color_result_to_index(&color_result), 4782968);
    }

    #[test]
    fn test_get_color_result() {
        use Color::*;

        let handle = Handle::from_string("2235m345p345888s4m");
        let other = Handle::from_string("2245567789m123p3m");
        let color_result = parse_color_result("ggygnnnnnnnnyy");
        println!("{:?}", handle.get_color_result(&other));
        assert_eq!(handle.get_color_result(&other), color_result);

        let handle = Handle {
            hand: [1, 1, 2, 4, 11, 12, 13, 20, 21, 22, 25, 25, 25, 3],
            pool: [false; 34],
            flags: 0,
        };
        let other = Handle {
            hand: [2, 3, 4, 6, 11, 12, 13, 20, 21, 22, 23, 24, 25, 6],
            pool: [false; 34],
            flags: 0,
        };
        let color_result = [
            Yellow, Yellow, Yellow, None, Green, Green, Green, Green, Green, Green, None, None,
            Green, None,
        ];
        println!("{:?}", handle.get_color_result(&other));
        assert_eq!(handle.get_color_result(&other), color_result);

        let handle = Handle {
            hand: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 1],
            pool: [false; 34],
            flags: 0,
        };
        let other = Handle {
            hand: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 1],
            pool: [false; 34],
            flags: 0,
        };
        let color_result = [
            Green, Green, Green, Green, Green, Green, Green, Green, Green, Green, Green, Green,
            Green, Green,
        ];
        assert_eq!(handle.get_color_result(&other), color_result);

        let handle = Handle {
            hand: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 1],
            pool: [false; 34],
            flags: 0,
        };
        let other = Handle {
            hand: [1, 3, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 1],
            pool: [false; 34],
            flags: 0,
        };
        let color_result = [
            Green, Yellow, Yellow, Green, Green, Green, Green, Green, Green, Green, Green, Green,
            Green, Green,
        ];
        assert_eq!(handle.get_color_result(&other), color_result);
    }

    #[test]
    fn test_match_color_result() {
        use Color::*;

        let handle = Handle {
            hand: [1, 1, 2, 4, 11, 12, 13, 20, 21, 22, 25, 25, 25, 3],
            pool: [false; 34],
            flags: 0,
        };
        let other = Handle {
            hand: [2, 3, 4, 6, 11, 12, 13, 20, 21, 22, 23, 24, 25, 6],
            pool: [false; 34],
            flags: 0,
        };
        let color_result = [
            Yellow, Yellow, Yellow, None, Green, Green, Green, Green, Green, Green, None, None,
            Green, None,
        ];
        assert!(handle.match_color_result(&other, &color_result));

        let handle = Handle {
            hand: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 1],
            pool: [false; 34],
            flags: 0,
        };
        let other = Handle {
            hand: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 1],
            pool: [false; 34],
            flags: 0,
        };
        let color_result = [
            Green, Green, Green, Green, Green, Green, Green, Green, Green, Green, Green, Green,
            Green, Green,
        ];
        assert!(handle.match_color_result(&other, &color_result));

        let handle = Handle {
            hand: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 1],
            pool: [false; 34],
            flags: 0,
        };
        let other = Handle {
            hand: [1, 3, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 1],
            pool: [false; 34],
            flags: 0,
        };
        let color_result = [
            Green, Yellow, Yellow, Green, Green, Green, Green, Green, Green, Green, Green, Green,
            Green, Green,
        ];
        assert!(handle.match_color_result(&other, &color_result));

        let handle = Handle {
            hand: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 1],
            pool: [false; 34],
            flags: 0,
        };
        let other = Handle {
            hand: [14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 14],
            pool: [false; 34],
            flags: 0,
        };
        let color_result = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        ];
        assert!(handle.match_color_result(&other, &color_result));
    }

    #[test]
    fn test_match_context() {
        let flags = store::MASK_TRUE_ALWAYS;
        let context = Context::parse_context("t");
        let handle = Handle {
            hand: [0; 14],
            pool: [false; 34],
            flags,
        };
        assert!(handle.match_context(&context));

        let flags = store::MASK_FALSE_IF_RON;
        let context = Context::parse_context("t");
        let handle = Handle {
            hand: [0; 14],
            pool: [false; 34],
            flags,
        };
        assert!(!handle.match_context(&context));

        let flags = 0b00000001;
        let context = Context::parse_context("");
        let handle = Handle {
            hand: [0; 14],
            pool: [false; 34],
            flags,
        };
        assert!(!handle.match_context(&context));
    }
}
