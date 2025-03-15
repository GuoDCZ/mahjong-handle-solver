// use mahc::calc::get_yaku_han;
// use mahc::hand;
use indicatif::{ProgressBar, ProgressStyle};
use mahjong_handle_solver::store;
use mahjong_handle_solver::utils::STYLE;
use mahjong_handle_solver::{generator::Generator, handle::Handle};
use std::collections::HashMap;
use std::io::prelude::*;
use std::{fs::File, io::Write};

fn generate_data(pb: ProgressBar) {
    pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
    const TASK: &str = "Generating cache file... ";
    println!("Cache file not found, generating...");
    let mut file = File::create("data").unwrap();
    Generator::new().for_each(|handle| {
        file.write_all(&handle.to_u128().to_be_bytes()).unwrap();
        pb.set_message(format!("{} {}", Handle::handle_to_string(&handle), TASK));
        pb.inc(1);
    });
    pb.finish_with_message(TASK.to_string() + "done");
}

fn refine_data(pb: ProgressBar, mut infile: File) {
    pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
    const TASK: &str = "Refining cache file... ";
    let mut handtable: HashMap<[u8; 14], u128> = HashMap::new();
    let mut buffer = [0u8; 16];
    loop {
        match infile.read(&mut buffer) {
            Ok(16) => {
                pb.inc(1);
                // if rand::thread_rng().gen_range(0..TOTAL) < SAMPLE_SIZE {
                let handle = Handle::from_u128(u128::from_be_bytes(buffer));
                let hand = handle.hand;
                if let Some(&raw) = handtable.get(&hand) {
                    let flags = handle.flags;
                    let flags_old = Handle::from_u128(raw).flags;
                    pb.set_message(format!(
                        "{} {}/{}",
                        Handle::hand_to_string(&hand),
                        flags,
                        flags_old
                    ));
                    if flags & store::MASK_TRUE_ALWAYS != 0
                        || (flags & store::MASK_FALSE_IF_RON == 0
                            && flags_old & store::MASK_FALSE_IF_RON != 0)
                    {
                        handtable.insert(hand, handle.to_u128());
                    }
                } else {
                    handtable.insert(hand, handle.to_u128());
                }
                // Handle::handle_to_string(&handle);
                // pb.set_message(format!("{} {}", Handle::handle_to_string(&handle), TASK));
                // pb.set_message(format!(" {}",  TASK));
                //     tx.send(handle).unwrap();
                // }
                // tx.send(handle).unwrap();
            }
            Ok(0) => break,
            _ => panic!("read error"),
        }
    }
    let mut outfile = File::create("data2").unwrap();
    for (_, &raw) in handtable.iter() {
        outfile.write_all(&raw.to_be_bytes()).unwrap();
    }
    pb.finish_with_message(TASK.to_string() + &"done");
}

fn load_data(pb: ProgressBar, mut file: File) -> impl Iterator<Item = Handle> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
        const TASK: &str = "Loading cache file... ";
        let mut buffer = [0u8; 16];
        loop {
            match file.read(&mut buffer) {
                Ok(16) => {
                    pb.inc(1);
                    let handle = Handle::from_u128(u128::from_be_bytes(buffer));
                    // pb.set_message(format!("{} {}", Handle::handle_to_string(&handle), TASK));
                    tx.send(handle).unwrap();
                }
                Ok(0) => break,
                _ => panic!("read error"),
            }
        }
        pb.finish_with_message(TASK.to_string() + &"done");
    });
    rx.into_iter()
}

fn main() {
    // If "data" exists, load it. Otherwise, generate it.

    const FILENAME: &str = "data";

    let multi = indicatif::MultiProgress::new();
    let file = File::open(FILENAME).unwrap();
    let len = file.metadata().unwrap().len() / size_of::<u128>() as u64;

    let g = {
        let pb = multi.add(ProgressBar::new(len));
        load_data(pb, file)
    };

    // Calculate the entropy of each handle.
    let pb = multi.add(ProgressBar::new(len));
    // let result = mahjong_handle_solver::mahd_fast::mahd_fast(pb, g);
    let result = mahjong_handle_solver::mahd_fast::mahd_fast2(pb, g);
    for (handle, entropy) in result {
        println!("{} {}", Handle::handle_to_string(&handle), entropy);
    }
}

#[cfg(test)]
mod tests {}
