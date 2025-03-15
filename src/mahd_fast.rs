use indicatif::{ProgressBar, ProgressStyle};

use super::handle::Handle;
use super::utils::STYLE;

// Pipeline:
// [Handle; 133770696]
//  -> [[u32; 34]; 34]
//  -> [[[u32; 34]; 34]; 13]
//
// -> [[[[u32; 9]; 34]; 34]; 13]
//
// -> [[[f64; 34]; 34]; 13]
//
// -> [f64; 133770696]
//
// -> f64
pub fn mahd_fast2(pb: ProgressBar, g: impl Iterator<Item = Handle>) -> Vec<(Handle, f64)> {
    let mut double_yellow_map = [[0; 34]; 34];
    let mut yellow_map = [0; 34];
    let mut double_green_map = [[[0; 34]; 34]; 13];
    let mut green_map = [[0; 34]; 14];
    let mut green_yellow_map = [[[0; 34]; 34]; 13];
    let mut yellow_green_map = [[[0; 34]; 34]; 13];

    pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
    pb.set_message("Register Handle");

    let mut handles: Vec<Handle> = vec![];
    for handle in g {
        pb.inc(1);
        for i in 0..34 {
            for j in i..34 {
                double_yellow_map[i][j] += (handle.pool[i] && handle.pool[j]) as u32;
            }
            yellow_map[i] += handle.pool[i] as u32;
        }
        for t in 0..14 {
            green_map[t][handle.hand[t] as usize] += 1;
        }
        for t in 0..13 {
            for i in 0..34 {
                for j in 0..34 {
                    green_yellow_map[t][i][j] +=
                        (handle.hand[t] == i as u8 && handle.pool[j]) as u32;
                    yellow_green_map[t][i][j] +=
                        (handle.pool[i] && handle.hand[t + 1] == j as u8) as u32;
                }
            }
            double_green_map[t][handle.hand[t] as usize][handle.hand[t + 1] as usize] += 1;
        }
        handles.push(handle);
    }

    for i in 0..34 {
        for j in i..34 {
            double_yellow_map[j][i] = double_yellow_map[i][j];
        }
    }

    let size = handles.len();
    let mut category_map = [[[[0; 9]; 34]; 34]; 14];
    for t in 0..13 {
        for i in 0..34 {
            for j in 0..34 {
                // [GG, GY, YG, GB, BG, YY, BY, YB, BB]
                category_map[t][i][j][0] = double_green_map[t][i][j];
                category_map[t][i][j][1] = green_yellow_map[t][i][j] - double_green_map[t][i][j];
                category_map[t][i][j][2] = yellow_green_map[t][i][j] - double_green_map[t][i][j];
                category_map[t][i][j][3] = green_map[t][i] - green_yellow_map[t][i][j];
                category_map[t][i][j][4] = green_map[t + 1][j] - yellow_green_map[t][i][j];
                category_map[t][i][j][5] = double_yellow_map[i][j] + double_green_map[t][i][j]
                    - green_yellow_map[t][i][j]
                    - yellow_green_map[t][i][j];
                category_map[t][i][j][6] =
                    yellow_map[i] - double_yellow_map[i][j] - category_map[t][i][j][3];
                category_map[t][i][j][7] =
                    yellow_map[j] - double_yellow_map[i][j] - category_map[t][i][j][4];
                category_map[t][i][j][8] =
                    size as u32 + double_yellow_map[i][j] - yellow_map[i] - yellow_map[j];
            }
        }
    }

    let mut entropy_map = [[[0.0; 34]; 34]; 14];
    let f_entropy = |x: u32| -> f64 {
        if x > 0 {
            let p = x as f64 / size as f64;
            -p * p.log2()
        } else {
            0.0
        }
    };
    for t in 0..14 {
        for i in 0..34 {
            for j in 0..34 {
                let mut entropy = 0.0;
                for k in 0..9 {
                    entropy += f_entropy(category_map[t][i][j][k]);
                }
                entropy_map[t][i][j] = entropy;
            }
        }
    }

    pb.set_position(0);
    const MAX_BUFFER_SIZE: usize = 10;
    let mut max_handles = Vec::with_capacity(MAX_BUFFER_SIZE);
    for handle in handles {
        pb.set_message(format!(
            "{} Finding Entropy",
            Handle::handle_to_string(&handle)
        ));
        pb.inc(1);
        let mut entropy = 0.0;
        for i in 0..13 {
            entropy += entropy_map[i][handle.hand[i] as usize][handle.hand[i + 1] as usize];
        }
        if max_handles.len() < MAX_BUFFER_SIZE {
            max_handles.push((handle, entropy));
        } else {
            let mut min_entropy = entropy;
            let mut min_index = 0;
            for i in 0..MAX_BUFFER_SIZE {
                if max_handles[i].1 < min_entropy {
                    min_entropy = max_handles[i].1;
                    min_index = i;
                }
            }
            if entropy > min_entropy {
                max_handles[min_index] = (handle, entropy);
            }
        }
    }
    pb.finish_with_message("Done");

    max_handles
}

// Pipeline:
// [Handle; 133770696]
//  -> [u32; 34]
//  -> [[u32; 34]; 14]
//
// -> [[(u32, u32); 34]; 14]
//
// -> [[f64; 34]; 14]
//
// -> [f64; 133770696]
//
// -> f64
pub fn mahd_fast(pb: ProgressBar, g: impl Iterator<Item = Handle>) -> Vec<(Handle, f64)> {
    let mut yellow_map = [0; 34];
    let mut green_map = [[0; 34]; 14];

    pb.set_style(ProgressStyle::with_template(STYLE).unwrap());

    let mut handles: Vec<Handle> = vec![];
    for handle in g {
        pb.set_message(format!(
            "{} Register Handle",
            Handle::handle_to_string(&handle)
        ));
        pb.inc(1);
        for i in 0..34 {
            yellow_map[i] += handle.pool[i] as u32;
        }
        for i in 0..14 {
            green_map[i][handle.hand[i] as usize] += 1;
        }
        handles.push(handle);
    }

    let size = handles.len();
    let mut category_map = [[(0, 0, 0); 34]; 14];
    for i in 0..14 {
        for j in 0..34 {
            category_map[i][j] = (
                green_map[i][j],
                yellow_map[j] - green_map[i][j],
                size as u32 - yellow_map[j],
            );
        }
    }

    let mut entropy_map = [[0.0; 34]; 14];
    let f_entropy = |x: u32| -> f64 {
        if x > 0 {
            let p = x as f64 / size as f64;
            -p * p.log2()
        } else {
            0.0
        }
    };
    for i in 0..14 {
        for j in 0..34 {
            let (a, b, c) = category_map[i][j];
            entropy_map[i][j] = f_entropy(a) + f_entropy(b) + f_entropy(c);
        }
    }

    pb.set_position(0);
    const MAX_BUFFER_SIZE: usize = 10;
    let mut max_handles = Vec::with_capacity(MAX_BUFFER_SIZE);
    for handle in handles {
        pb.set_message(format!(
            "{} Finding Entropy",
            Handle::handle_to_string(&handle)
        ));
        pb.inc(1);
        let mut entropy = 0.0;
        for i in 0..14 {
            entropy += entropy_map[i][handle.hand[i] as usize];
        }
        if max_handles.len() < MAX_BUFFER_SIZE {
            max_handles.push((handle, entropy));
        } else {
            let mut min_entropy = entropy;
            let mut min_index = 0;
            for i in 0..MAX_BUFFER_SIZE {
                if max_handles[i].1 < min_entropy {
                    min_entropy = max_handles[i].1;
                    min_index = i;
                }
            }
            if entropy > min_entropy {
                max_handles[min_index] = (handle, entropy);
            }
        }
    }
    pb.finish_with_message("Done");

    max_handles
}
