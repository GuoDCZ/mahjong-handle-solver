use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::u8;

use super::handle::{Hand, Handle};

const MAX_TILE: usize = 34;
const MAX_POS: usize = 14;
const MAX_PAIR_POS: usize = 13;

const TILES: Range<usize> = 0..MAX_TILE;
const POSES: Range<usize> = 0..MAX_POS;
const PAIR_POSES: Range<usize> = 0..MAX_PAIR_POS;

struct MetaMap {
    gg: [[[u32; 34]; 34]; 13],
    gy: [[[u32; 34]; 34]; 13],
    yg: [[[u32; 34]; 34]; 13],
    g: [[u32; 34]; 14],
    yy: [[u32; 34]; 34],
    y: [u32; 34],
    total: u32,
}

impl MetaMap {
    fn new() -> Self {
        MetaMap {
            gg: [[[0; 34]; 34]; 13],
            gy: [[[0; 34]; 34]; 13],
            yg: [[[0; 34]; 34]; 13],
            g: [[0; 34]; 14],
            yy: [[0; 34]; 34],
            y: [0; 34],
            total: 0,
        }
    }

    fn register(mut self, handle: &Handle) -> Self {
        for pos in PAIR_POSES {
            self.gg[pos][handle.hand[pos] as usize][handle.hand[pos + 1] as usize] += 1;
        }
        for pos in PAIR_POSES {
            for fst in TILES {
                for snd in TILES {
                    self.gy[pos][fst][snd] +=
                        (handle.hand[pos] == fst as u8 && handle.pool[snd]) as u32;
                    self.yg[pos][fst][snd] +=
                        (handle.pool[fst] && handle.hand[pos + 1] == snd as u8) as u32;
                }
            }
        }
        for pos in POSES {
            self.g[pos][handle.hand[pos] as usize] += 1;
        }
        for fst in TILES {
            // Note: This is a symmetric matrix. For the purpose of efficiency, we only
            // store half of the matrix. Use `flip_yy` to complete the matrix before
            // using it.
            for snd in fst..MAX_TILE {
                self.yy[fst][snd] += (handle.pool[fst] && handle.pool[snd]) as u32;
            }
        }
        for tile in TILES {
            self.y[tile] += handle.pool[tile] as u32;
        }
        self.total += 1;
        self
    }

    fn flip_yy(mut self) -> Self {
        for fst in TILES {
            for snd in (fst + 1)..MAX_TILE {
                self.yy[snd][fst] = self.yy[fst][snd];
            }
        }
        self
    }
}

impl std::ops::Add for MetaMap {
    type Output = Self;
    fn add(mut self, other: Self) -> Self {
        for pos in PAIR_POSES {
            for fst in TILES {
                for snd in TILES {
                    self.gg[pos][fst][snd] += other.gg[pos][fst][snd];
                    self.gy[pos][fst][snd] += other.gy[pos][fst][snd];
                    self.yg[pos][fst][snd] += other.yg[pos][fst][snd];
                }
            }
        }
        for pos in POSES {
            for tile in TILES {
                self.g[pos][tile] += other.g[pos][tile];
            }
        }
        for fst in TILES {
            for snd in TILES {
                self.yy[fst][snd] += other.yy[fst][snd];
            }
        }
        for tile in TILES {
            self.y[tile] += other.y[tile];
        }
        self.total += other.total;
        self
    }
}

enum PairColor {
    GG,
    GY,
    YG,
    YY,
    GN,
    NG,
    YN,
    NY,
    NN,
}

const VARIANT_COUNT_PAIR_COLOR: usize = 9;

type CatagoryMap = (
    [[[[u32; VARIANT_COUNT_PAIR_COLOR]; MAX_TILE]; MAX_TILE]; MAX_PAIR_POS],
    u32,
);

fn mk_catagory(meta_map: MetaMap) -> CatagoryMap {
    let mut map = [[[[0; VARIANT_COUNT_PAIR_COLOR]; MAX_TILE]; MAX_TILE]; MAX_PAIR_POS];

    for pos in PAIR_POSES {
        for fst in TILES {
            for snd in TILES {
                use PairColor::*;
                let pair_map = &mut map[pos][fst][snd];
                pair_map[GG as usize] = meta_map.gg[pos][fst][snd];
                pair_map[GY as usize] = meta_map.gy[pos][fst][snd] - meta_map.gg[pos][fst][snd];
                pair_map[YG as usize] = meta_map.yg[pos][fst][snd] - meta_map.gg[pos][fst][snd];
                pair_map[YY as usize] = meta_map.yy[fst][snd] + meta_map.gg[pos][fst][snd]
                    - meta_map.gy[pos][fst][snd]
                    - meta_map.yg[pos][fst][snd];
                pair_map[GN as usize] = meta_map.g[pos][fst] - meta_map.gy[pos][fst][snd];
                pair_map[NG as usize] = meta_map.g[pos + 1][snd] - meta_map.yg[pos][fst][snd];
                pair_map[YN as usize] =
                    meta_map.y[fst] - meta_map.yy[fst][snd] - pair_map[GN as usize];
                pair_map[NY as usize] =
                    meta_map.y[snd] - meta_map.yy[fst][snd] - pair_map[NG as usize];
                pair_map[NN as usize] =
                    meta_map.total + meta_map.yy[fst][snd] - meta_map.y[fst] - meta_map.y[snd];
            }
        }
    }

    (map, meta_map.total)
}

type EntropyMap = [[[f64; MAX_TILE]; MAX_TILE]; MAX_PAIR_POS];

fn mk_entropy((catagory_map, total): CatagoryMap) -> EntropyMap {
    let f_entropy = |x: &u32| -> f64 {
        if *x > 0 {
            let p = *x as f64 / total as f64;
            -p * p.log2()
        } else {
            0.0
        }
    };
    catagory_map.map(|m| m.map(|m| m.map(|color| color.iter().map(f_entropy).sum::<f64>())))
}

fn find_entropy(entropy_map: &EntropyMap, handle: &Handle) -> f64 {
    let mut entropy = 0.0;
    for pos in PAIR_POSES {
        entropy += entropy_map[pos][handle.hand[pos] as usize][handle.hand[pos + 1] as usize];
    }
    entropy
}

pub fn mahd_fast2_prepare(inc: impl Fn(), hs: &Vec<Handle>) -> EntropyMap {
    let mut meta_map = MetaMap::new();
    for handle in hs {
        inc();
        meta_map = meta_map.register(&handle);
    }
    let meta_map = meta_map.flip_yy();
    let catagory_map = mk_catagory(meta_map);
    mk_entropy(catagory_map)
}

#[derive(Debug)]
pub struct HandleEntropy {
    pub hand: Hand,
    pub entropy: f64,
}

impl PartialEq for HandleEntropy {
    fn eq(&self, other: &Self) -> bool {
        self.entropy == other.entropy
    }
}

impl Eq for HandleEntropy {}

impl PartialOrd for HandleEntropy {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.entropy.partial_cmp(&other.entropy)
    }
}

impl Ord for HandleEntropy {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.entropy.partial_cmp(&other.entropy).unwrap()
    }
}

fn top_n_biggest<T: Ord>(inc: impl Fn(), hes: Vec<T>, size: usize) -> Vec<T> {
    let mut heap = BinaryHeap::new();
    for item in hes {
        inc();
        heap.push(Reverse(item));
        if heap.len() > size {
            heap.pop();
        }
    }
    heap.into_iter().map(|he| he.0).collect()
}

pub fn mahd_fast2_entropy(
    inc: impl Fn(),
    hs_all: &Vec<Handle>,
    entropy_map: &EntropyMap,
) -> Vec<HandleEntropy> {
    hs_all
        .iter()
        .map(|handle| {
            inc();
            HandleEntropy {
                hand: handle.hand.clone(),
                entropy: find_entropy(entropy_map, handle),
            }
        })
        .collect::<Vec<_>>()
}

pub fn mahd_fast2(inc: impl Fn(), hes: Vec<HandleEntropy>, size: usize) -> Vec<Handle> {
    top_n_biggest(inc, hes, size)
        .into_iter()
        .map(|he| Handle {
            hand: he.hand,
            pool: [false; 34],
            flags: 0,
        })
        .collect()
}

pub fn mahd_killer_prepare(
    inc: impl Fn(),
    handles: &Vec<Handle>,
    handles_all: &Vec<Handle>,
) -> Vec<HandleEntropy> {
    let total = handles.len() as u32;
    handles_all
        .iter()
        .map(|guess| {
            inc();
            HandleEntropy {
                hand: guess.hand.clone(),
                entropy: handles
                    .iter()
                    .fold(HashMap::new(), |mut acc, key| {
                        let result = key.get_color_result(&guess);
                        *acc.entry(result).or_insert(0) += 1;
                        acc
                    })
                    .iter()
                    .fold(0.0, |acc, (_, &count)| {
                        let p = count as f64 / total as f64;
                        acc - p * p.log2()
                    }),
            }
        })
        .collect()
}

pub fn mahd_killer(inc: impl Fn(), hes: Vec<HandleEntropy>, size: usize) -> Vec<HandleEntropy> {
    top_n_biggest(inc, hes, size)
}

pub fn mahd_killer_inner(handles: &Vec<Handle>) -> Option<Handle> {
    for guess in handles {
        let mut acc = HashSet::new();
        let mut fail = false;
        for key in handles {
            let result = key.get_color_result(&guess);
            if !acc.insert(result) {
                fail = true;
                break;
            }
        }
        if !fail {
            return Some(*guess);
        }
    }
    None
}
