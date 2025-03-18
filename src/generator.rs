use super::handle::Handle;
use super::store;
use super::utils::{koutsu_of_tile, next_tile, shuntsu_of_tile, toitsu_of_tile};
use riichi::agenda::AgendaName;
use riichi::hand::PartitionedHand;
use riichi::score::Score;
use riichi::tile::Tile;
use std::iter::Iterator;
use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Clone)]
pub struct Finder {
    partitions: PartitionedHand,
    pool: [u8; 34],
    winning_tile: Option<Tile>,
    toitsu: Option<Tile>,
    nmentsu: u8,
    stage: FinderStage,
    curr: Tile,
}

#[derive(Clone)]
pub enum FinderStage {
    Toitsu,
    Koutsu,
    Shuntsu,
    Check,
}

impl Finder {
    pub fn new() -> Self {
        Finder {
            partitions: PartitionedHand {
                group_items: vec![],
                is_singular_wait: false,
            },
            pool: [0; 34],
            winning_tile: None,
            toitsu: None,
            nmentsu: 0,
            stage: FinderStage::Toitsu,
            curr: Tile::_1m,
        }
    }

    fn curr_num(&self) -> u8 {
        self.pool[self.curr as usize]
    }

    fn need_toitsu(&self) -> bool {
        self.toitsu.is_none()
    }

    fn need_mentsu(&self) -> bool {
        self.nmentsu != 4
    }

    fn finished(&self) -> bool {
        !self.need_mentsu() && !self.need_toitsu() && self.winning_tile.is_some()
    }

    pub fn next(self, tx: &Sender<Handle>) {
        match self.stage {
            FinderStage::Toitsu => {
                if self.need_toitsu() && self.curr_num() <= 2 {
                    let mut finder = self.clone();
                    finder.stage = FinderStage::Shuntsu;
                    finder
                        .partitions
                        .group_items
                        .push((toitsu_of_tile(finder.curr), false));
                    finder.toitsu = Some(finder.curr);
                    finder.pool[finder.curr as usize] += 2;
                    if finder.winning_tile.is_none() {
                        let mut finder = finder.clone();
                        finder.winning_tile = Some(finder.curr);
                        finder.partitions.is_singular_wait = true;
                        finder.next(tx);
                    }
                    finder.next(tx);
                }
                Finder {
                    stage: FinderStage::Koutsu,
                    ..self
                }
                .next(tx);
            }
            FinderStage::Koutsu => {
                if self.need_mentsu() && self.curr_num() <= 1 {
                    let mut finder = self.clone();
                    finder.stage = FinderStage::Shuntsu;
                    finder.nmentsu += 1;
                    finder.pool[finder.curr as usize] += 3;
                    if finder.winning_tile.is_none() {
                        let mut finder = finder.clone();
                        finder
                            .partitions
                            .group_items
                            .push((koutsu_of_tile(finder.curr), true));
                        finder.winning_tile = Some(finder.curr);
                        finder.next(tx);
                    }
                    finder
                        .partitions
                        .group_items
                        .push((koutsu_of_tile(finder.curr), false));
                    finder.next(tx);
                }
                Finder {
                    stage: FinderStage::Shuntsu,
                    ..self
                }
                .next(tx);
            }
            FinderStage::Shuntsu => {
                let shuntsu = shuntsu_of_tile(self.curr);
                if self.need_mentsu() && self.curr_num() <= 3 && shuntsu.is_some() {
                    let mut finder = self.clone();
                    finder
                        .partitions
                        .group_items
                        .push((shuntsu.unwrap(), false));
                    finder.nmentsu += 1;
                    finder.pool[finder.curr as usize] += 1;
                    finder.pool[finder.curr as usize + 1] += 1;
                    finder.pool[finder.curr as usize + 2] += 1;
                    if finder.winning_tile.is_none() {
                        let mut finder1 = finder.clone();
                        finder1.winning_tile = Some(finder1.curr);
                        finder1.partitions.is_singular_wait =
                            [Tile::_1m, Tile::_1p, Tile::_1s].contains(&finder1.curr);
                        finder1.next(tx);
                        let mut finder2 = finder.clone();
                        finder2.winning_tile = Some(next_tile(finder2.curr).unwrap());
                        finder2.partitions.is_singular_wait = true;
                        finder2.next(tx);
                        let mut finder3 = finder.clone();
                        finder3.winning_tile =
                            Some(next_tile(next_tile(finder3.curr).unwrap()).unwrap());
                        finder3.partitions.is_singular_wait =
                            [Tile::_1m, Tile::_1p, Tile::_1s].contains(&finder3.curr);
                        finder3.next(tx);
                    }
                    finder.next(tx);
                }
                Finder {
                    stage: FinderStage::Check,
                    ..self
                }
                .next(tx);
            }
            FinderStage::Check => {
                if self.finished() {
                    let handle = self.to_handle();
                    tx.send(handle).unwrap();
                } else {
                    match next_tile(self.curr) {
                        Some(tile) => {
                            Finder {
                                curr: tile,
                                stage: FinderStage::Toitsu,
                                ..self
                            }
                            .next(tx);
                        }
                        None => {}
                    }
                }
            }
        }
    }

    fn get_flags(partitions: PartitionedHand) -> u8 {
        let mut flags: u8 = 0;
        match partitions.calculate_score(riichi::agendas_template::AGENDAS_TEMPLATE) {
            Score::Done => flags |= store::MASK_TRUE_ALWAYS,
            Score::Name(agenda_names) => {
                let have_pinfu = agenda_names.contains(&AgendaName::Pinfu);
                let mut have_pinfu_pending_jantou = false;
                for name in &agenda_names {
                    match name {
                        AgendaName::YakuhaiTon => flags |= store::ANY_TON,
                        AgendaName::YakuhaiNan => flags |= store::ANY_NAN,
                        AgendaName::YakuhaiShaa => flags |= store::ANY_SHA,
                        AgendaName::YakuhaiPei => flags |= store::ANY_PEI,
                        AgendaName::YakuhaiJantouTon => {
                            if have_pinfu {
                                have_pinfu_pending_jantou = true;
                                flags |= store::NOT_ANY_TON;
                            }
                        }
                        AgendaName::YakuhaiJantouNan => {
                            if have_pinfu {
                                have_pinfu_pending_jantou = true;
                                flags |= store::NOT_ANY_NAN;
                            }
                        }
                        AgendaName::YakuhaiJantouShaa => {
                            if have_pinfu {
                                have_pinfu_pending_jantou = true;
                                flags |= store::NOT_ANY_SHA;
                            }
                        }
                        AgendaName::YakuhaiJantouPei => {
                            if have_pinfu {
                                have_pinfu_pending_jantou = true;
                                flags |= store::NOT_ANY_PEI;
                            }
                        }
                        _ => {}
                    }
                }
                if have_pinfu {
                    if !have_pinfu_pending_jantou {
                        flags |= store::MASK_TRUE_ALWAYS;
                    }
                } else {
                    if flags & store::MASK_GROUP_ANY == 0 {
                        flags |= store::MASK_FALSE_IF_RON;
                    }
                }
            }
        }
        flags
    }

    pub fn to_handle(mut self) -> Handle {
        // Handle.pool
        let mut pool = [false; 34];
        for i in 0..34 {
            pool[i] = self.pool[i] != 0;
        }

        // Handle.hand
        let mut hand = [0; 14];
        let winning_tile = self.winning_tile.unwrap();
        self.pool[winning_tile as usize] -= 1;
        let mut waiting_hand_index = 0;
        for i in 0..34 {
            for _ in 0..self.pool[i] {
                hand[waiting_hand_index] = i as u8;
                waiting_hand_index += 1;
            }
        }
        assert!(waiting_hand_index == 13);
        hand[13] = winning_tile as u8;

        // Handle.flags
        let flags = Self::get_flags(self.partitions);
        Handle { hand, pool, flags }
    }
}

#[derive(Clone)]
pub struct ChiitoiFinder {
    hand: [u8; 14],
    pool: [bool; 34],
    npairs: usize,
    curr: u8,
}

impl ChiitoiFinder {
    pub fn new() -> Self {
        ChiitoiFinder {
            hand: [0; 14],
            pool: [false; 34],
            npairs: 0,
            curr: 0,
        }
    }

    fn finished(&self) -> bool {
        self.npairs == 7
    }

    pub fn next(mut self, tx: &Sender<Handle>) {
        if self.finished() {
            let mut handle = Handle {
                hand: self.hand,
                pool: self.pool,
                flags: store::MASK_TRUE_ALWAYS,
            };
            tx.send(handle.clone()).unwrap();
            for i in 1..6 {
                handle.hand.swap(13 - 2 * i, 13);
                tx.send(handle.clone()).unwrap();
            }
        } else if self.curr < 34 {
            ChiitoiFinder {
                curr: self.curr + 1,
                ..self
            }
            .next(tx);
            self.hand[self.npairs * 2] = self.curr;
            self.hand[self.npairs * 2 + 1] = self.curr;
            self.npairs += 1;
            self.pool[self.curr as usize] = true;
            self.curr += 1;
            self.next(tx);
        }
    }
}

#[derive(Clone)]
pub struct KokushiFinder {
    hand: [u8; 14],
    pool: [bool; 34],
}

impl KokushiFinder {
    pub fn new() -> Self {
        let mut pool = [false; 34];
        pool[Tile::_1m as usize] = true;
        pool[Tile::_9m as usize] = true;
        pool[Tile::_1p as usize] = true;
        pool[Tile::_9p as usize] = true;
        pool[Tile::_1s as usize] = true;
        pool[Tile::_9s as usize] = true;
        pool[Tile::_1z as usize] = true;
        pool[Tile::_2z as usize] = true;
        pool[Tile::_3z as usize] = true;
        pool[Tile::_4z as usize] = true;
        pool[Tile::_5z as usize] = true;
        pool[Tile::_6z as usize] = true;
        pool[Tile::_7z as usize] = true;
        let hand = [
            Tile::_1m as u8,
            Tile::_1m as u8,
            Tile::_9m as u8,
            Tile::_1p as u8,
            Tile::_9p as u8,
            Tile::_1s as u8,
            Tile::_9s as u8,
            Tile::_1z as u8,
            Tile::_2z as u8,
            Tile::_3z as u8,
            Tile::_4z as u8,
            Tile::_5z as u8,
            Tile::_6z as u8,
            Tile::_7z as u8,
        ];
        KokushiFinder { pool, hand }
    }

    fn next_helper(self, tx: &Sender<Handle>) {
        let mut handle = Handle {
            hand: self.hand,
            pool: self.pool,
            flags: store::MASK_TRUE_ALWAYS,
        };
        tx.send(handle.clone()).unwrap();
        for i in 1..14 {
            handle.hand.swap(13 - i, 13);
            tx.send(handle.clone()).unwrap();
        }
    }

    pub fn next(self, tx: &Sender<Handle>) {
        for i in 1..13 {
            let mut finder = self.clone();
            finder.hand[i] = finder.hand[i + 1];
            finder.next_helper(tx);
        }
    }
}

pub struct Generator {
    rx: Receiver<Handle>,
}

impl Generator {
    // Use this to send partitioned hands to `next()` through channels
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let tx_ = tx.clone();
        std::thread::spawn(move || Finder::new().next(&tx_));
        let tx_ = tx.clone();
        std::thread::spawn(move || ChiitoiFinder::new().next(&tx_));
        let tx_ = tx.clone();
        std::thread::spawn(move || KokushiFinder::new().next(&tx_));
        Self { rx }
    }
}

impl Iterator for Generator {
    type Item = Handle;

    fn next(&mut self) -> Option<Self::Item> {
        self.rx.recv().ok()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use riichi::tile_group::{KoutsuGroup::*, ShuntsuGroup::*, TileGroup::*, ToitsuGroup::*};

    #[test]
    fn test_pinfu() {
        let partitions = PartitionedHand {
            group_items: vec![
                (Toitsu(_11m), false),
                (Shuntsu(_123m), false),
                (Shuntsu(_456m), false),
                (Shuntsu(_123p), false),
                (Shuntsu(_789s), false),
            ],
            is_singular_wait: false,
        };
        let flags = Finder::get_flags(partitions);
        assert_eq!(flags, store::MASK_TRUE_ALWAYS);

        let partitions = PartitionedHand {
            group_items: vec![
                (Toitsu(_11m), false),
                (Shuntsu(_123m), false),
                (Shuntsu(_456m), false),
                (Shuntsu(_123p), false),
                (Shuntsu(_789s), false),
            ],
            is_singular_wait: true,
        };
        let flags = Finder::get_flags(partitions);
        assert_eq!(flags, store::MASK_FALSE_IF_RON);

        let partitions = PartitionedHand {
            group_items: vec![
                (Toitsu(_66z), false),
                (Shuntsu(_123m), false),
                (Shuntsu(_456m), false),
                (Shuntsu(_123p), false),
                (Shuntsu(_789s), false),
            ],
            is_singular_wait: false,
        };
        let flags = Finder::get_flags(partitions);
        assert_eq!(flags, store::MASK_FALSE_IF_RON);

        let partitions = PartitionedHand {
            group_items: vec![
                (Toitsu(_11z), false),
                (Shuntsu(_123m), false),
                (Shuntsu(_456m), false),
                (Shuntsu(_123p), false),
                (Shuntsu(_789s), false),
            ],
            is_singular_wait: false,
        };
        let flags = Finder::get_flags(partitions);
        assert_eq!(flags, 0b00000000);

        let partitions = PartitionedHand {
            group_items: vec![
                (Toitsu(_44z), false),
                (Shuntsu(_123m), false),
                (Shuntsu(_456m), false),
                (Shuntsu(_123p), false),
                (Shuntsu(_789s), false),
            ],
            is_singular_wait: false,
        };
        let flags = Finder::get_flags(partitions);
        assert_eq!(flags, 0b00110000);
    }

    #[test]
    fn test_yakuhai() {
        let partitions = PartitionedHand {
            group_items: vec![
                (Toitsu(_11m), false),
                (Koutsu(_111z), false),
                (Koutsu(_222z), true),
                (Koutsu(_333z), false),
                (Shuntsu(_567s), false),
            ],
            is_singular_wait: false,
        };
        let flags = Finder::get_flags(partitions);
        assert_eq!(flags, 0b00000111);
    }
}
