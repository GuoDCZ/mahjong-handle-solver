use indicatif::{ProgressBar, ProgressStyle};

use super::handle::Handle;
use super::utils::STYLE;

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
