// use mahc::calc::get_yaku_han;
// use mahc::hand;
use indicatif::{ProgressBar, ProgressStyle};
use mahjong_handle_solver::mahd_fast2::mahd_killer_inner;
use mahjong_handle_solver::utils::STYLE;
use mahjong_handle_solver::{
    generator::Generator,
    handle::Handle,
    mahd_fast2::{
        mahd_fast2, mahd_fast2_entropy, mahd_fast2_prepare, mahd_killer, mahd_killer_prepare,
    },
};
use mahjong_handle_solver::{handle, store};
use std::collections::HashMap;
use std::env;
use std::io::prelude::*;
use std::mem::size_of;
use std::{fs::File, io::Write};

fn _generate_data(pb: ProgressBar) {
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

fn _refine_data(pb: ProgressBar, mut infile: File) {
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

fn _load_data(inc: impl Fn()) -> Vec<Handle> {
    let mut file = File::open("data").unwrap();
    let mut buffer = [0u8; 16];
    let mut hs = vec![];
    loop {
        match file.read(&mut buffer) {
            Ok(16) => {
                inc();
                hs.push(Handle::from_u128(u128::from_be_bytes(buffer)));
            }
            Ok(0) => break,
            _ => panic!("read error"),
        }
    }
    hs
}

macro_rules! call_with_progress {
    ($name:expr, $total:expr, $func:expr, $($args:expr),* $(,)?) => {{
        let pb = ProgressBar::new($total as u64);
        pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
        pb.set_message($name);
        let result = $func(&|| pb.inc(1), $($args,)*);
        pb.finish_with_message($name.to_string() + &" done");
        result
    }};
}

fn get_context() -> handle::Context {
    let args: Vec<String> = env::args().collect();
    let default = "".to_string();
    let arg = args.get(1).unwrap_or(&default);
    handle::Context::parse_context(&arg)
}

fn get_result() -> handle::ColorResult {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    handle::parse_color_result(&buffer)
}

fn load_index_file(result: &handle::ColorResult) -> (u32, u32) {
    let mut file = File::open("index").unwrap();
    let color_index = handle::color_result_to_index(result);
    file.seek(std::io::SeekFrom::Start((color_index - 1) as u64 * 4))
        .unwrap();
    let mut buffer = [0u8; 4]; // u32
    file.read(&mut buffer).unwrap();
    let index = u32::from_be_bytes(buffer);
    file.read(&mut buffer).unwrap();
    let index_end = u32::from_be_bytes(buffer);
    (index, index_end)
}

fn load_data_with_index(inc: impl Fn(), index: u32, index_end: u32) -> Vec<Handle> {
    let mut file = File::open("data").unwrap();
    file.seek(std::io::SeekFrom::Start(index as u64 * 16))
        .unwrap();
    let mut buffer = [0u8; 16]; // u128
    let mut hs = vec![];
    for _ in index..index_end {
        inc();
        file.read(&mut buffer).unwrap();
        hs.push(Handle::from_u128(u128::from_be_bytes(buffer)));
    }
    hs
}

fn load_data_all(inc: impl Fn()) -> Vec<Handle> {
    let mut file = File::open("data_all").unwrap();
    let mut buffer = [0u8; 15];
    let mut hs = vec![];
    loop {
        match file.read(&mut buffer) {
            Ok(15) => {
                inc();
                hs.push(Handle {
                    hand: buffer[0..14].try_into().unwrap(),
                    pool: [false; 34],
                    flags: buffer[14],
                })
            }
            Ok(0) => break,
            _ => panic!("read error"),
        }
    }
    hs
}

fn filter_context(inc: impl Fn(), hs: Vec<Handle>, context: &handle::Context) -> Vec<Handle> {
    hs.into_iter()
        .filter(|handle| {
            inc();
            handle.match_context(context)
        })
        .collect::<Vec<Handle>>()
}

fn main() {
    let context = get_context();

    // Provide best 1st guess
    let mut guess = Handle::best_1st();
    println!("[1] guess: {}", Handle::handle_to_string(&guess));

    // Collect the color results from std input
    print!("[1] result: ");
    std::io::stdout().flush().unwrap();
    let result = get_result();

    // Load index file
    let (index, index_end) = load_index_file(&result);

    // Generate from the cache file
    let hs = load_data_with_index(|| (), index, index_end);
    let mut hs = filter_context(|| (), hs, &context);

    // loading all data
    let len = File::open("data").unwrap().metadata().unwrap().len() / size_of::<u128>() as u64;
    let hs_all = call_with_progress!(
        "Loading all data",
        len, // len
        load_data_all,
    );

    let hs_all = call_with_progress!(
        "Filtering all context",
        hs_all.len(),
        filter_context,
        hs_all,
        &context
    );

    let mut round = 1;

    loop {
        guess = {
            let guess_opt = if hs.len() < 1000 {
                // finding inner killer
                mahd_killer_inner(&hs)
            } else {
                None
            };

            if let Some(guess) = guess_opt {
                guess
            } else {
                // finding best 2nd guess
                let entropy_map =
                    call_with_progress!("Preparing Entropy Map", hs.len(), mahd_fast2_prepare, &hs);

                let n_killer_candidate = 10000000 / hs.len();

                let hes = call_with_progress!(
                    "Finding Best Guess Entropy",
                    hs_all.len(),
                    mahd_fast2_entropy,
                    &hs_all,
                    &entropy_map,
                );

                let result = call_with_progress!(
                    "Finding Best Guess",
                    hes.len(),
                    mahd_fast2,
                    hes,
                    n_killer_candidate,
                );

                // finding killer
                let hes = call_with_progress!(
                    "Finding Killer Prepare",
                    n_killer_candidate,
                    mahd_killer_prepare,
                    &hs,
                    &result
                );

                let result =
                    call_with_progress!("Finding Killer", hes.len(), mahd_killer, hes, 10,);

                for handle in &result {
                    println!(
                        "{} {}",
                        Handle::hand_to_string(&handle.hand),
                        handle.entropy
                    );
                }

                Handle {
                    hand: result.iter().max().unwrap().hand,
                    pool: [false; 34],
                    flags: 0,
                }
            }
        };

        round += 1;

        println!("[{}] guess: {}", round, Handle::handle_to_string(&guess));

        print!("[{}] result: ", round);
        std::io::stdout().flush().unwrap();
        let result = get_result();

        hs = call_with_progress!("Filtering color result", hs.len(), |inc: &dyn Fn()
            -> ()| {
            inc();
            hs.into_iter()
                .filter(|handle| handle.match_color_result(&guess, &result))
                .collect::<Vec<Handle>>()
        },);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    // #[test]
    // fn test_data_revert() {
    //     const FILENAME: &str = "data";
    //     let mut file = File::open(FILENAME).unwrap();
    //     call_with_progress!(
    //         "Loading data",
    //         file.metadata().unwrap().len() / size_of::<u128>() as u64, // len
    //         |inc: &dyn Fn() -> ()| {
    //             let mut file_out = File::create("data_all").unwrap();
    //             let mut buffer = [0u8; 16];
    //             loop {
    //                 match file.read(&mut buffer) {
    //                     Ok(16) => {
    //                         inc();
    //                         let handle = Handle::from_u128(u128::from_be_bytes(buffer));
    //                         file_out.write_all(&handle.hand).unwrap();
    //                         file_out.write_all(&handle.flags.to_be_bytes()).unwrap();
    //                     }
    //                     Ok(0) => break,
    //                     _ => panic!("read error"),
    //                 }
    //             }
    //         },
    //     );
    // }

    // #[test]
    // fn test_data_partition() {
    //     const FILENAME: &str = "data";
    //     let file = File::open(FILENAME).unwrap();
    //     let hs = call_with_progress!(
    //         "Loading data",
    //         file.metadata().unwrap().len() / size_of::<u128>() as u64, // len
    //         _load_data,
    //     );

    //     const TOTAL_COLOR_RESULT: usize = 4782969;
    //     let hs_len = hs.len();

    //     let guess = Handle::best_1st();

    //     let sorted_data: Vec<std::collections::BTreeSet<Handle>> =
    //         call_with_progress!("Sorting data", hs.len(), |inc: &dyn Fn() -> ()| {
    //             let mut sorted_data: Vec<std::collections::BTreeSet<Handle>> =
    //                 vec![std::collections::BTreeSet::new(); TOTAL_COLOR_RESULT];
    //             hs.into_iter().for_each(|handle| {
    //                 inc();
    //                 let color_result = handle.get_color_result(&guess);
    //                 let index = handle::color_result_to_index(&color_result) as usize;
    //                 if color_result == handle::parse_color_result("yyynggggggnngn") {
    //                     println!("{}", Handle::handle_to_string(&handle));
    //                 }
    //                 if color_result == handle::parse_color_result("gggggggggggggg") {
    //                     println!("{}", Handle::handle_to_string(&handle));
    //                 }
    //                 sorted_data[index].insert(handle);
    //             });
    //             sorted_data
    //         },);
    //     // make index file
    //     call_with_progress!(
    //         "Making index file",
    //         TOTAL_COLOR_RESULT,
    //         |inc: &dyn Fn() -> ()| {
    //             let mut index_file = File::create("index").unwrap();
    //             let mut index: u32 = 0;
    //             sorted_data.iter().for_each(|data| {
    //                 inc();
    //                 index += data.len() as u32;
    //                 index_file.write_all(&index.to_be_bytes()).unwrap();
    //             });
    //             assert_eq!(index, hs_len as u32);
    //             index_file.flush().unwrap();
    //         },
    //     );

    //     // make data file
    //     call_with_progress!("Making data file", hs_len, |inc: &dyn Fn() -> ()| {
    //         let mut data_file = File::create("data2").unwrap();
    //         sorted_data.iter().for_each(|data| {
    //             data.iter().for_each(|handle| {
    //                 inc();
    //                 data_file
    //                     .write_all(&handle.to_u128().to_be_bytes())
    //                     .unwrap();
    //             });
    //         });
    //         data_file.flush().unwrap();
    //     },);
    // }

    #[test]
    fn test_index_distribution() {
        use csv::Writer;
        const FILENAME: &str = "index";
        let mut file = File::open(FILENAME).unwrap();
        let mut buffer = [0u8; 4];
        let mut index = 0;
        let mut distribution: Vec<u32> = vec![];
        loop {
            match file.read(&mut buffer) {
                Ok(4) => {
                    let index_new = u32::from_be_bytes(buffer);
                    distribution.push(index_new - index);
                    index = index_new;
                }
                Ok(0) => break,
                _ => panic!("read error"),
            }
        }
        let mut writer = Writer::from_path("distribution.csv").unwrap();
        distribution.iter().for_each(|count| {
            writer.write_record(&[count.to_string()]).unwrap();
        });
        writer.flush().unwrap();
    }

    #[test]
    fn test_handle_1() {
        use super::handle::Color::*;
        use super::handle::Context;
        use super::handle::Handle;
        let handle = Handle {
            hand: [1, 1, 2, 4, 11, 12, 13, 20, 21, 22, 25, 25, 25, 3],
            pool: [false; 34],
            flags: super::store::MASK_TRUE_ALWAYS,
        };
        println!("{}", Handle::handle_to_string(&handle));
        let other = Handle {
            hand: [2, 3, 4, 6, 11, 12, 13, 20, 21, 22, 23, 24, 25, 6],
            pool: [false; 34],
            flags: 0,
        };
        let context = Context::parse_context("e");
        let color_result = [
            Yellow, Yellow, Yellow, None, Green, Green, Green, Green, Green, Green, None, None,
            Green, None,
        ];
        [handle]
            .iter()
            .filter(|handle| handle.match_context(&context))
            .filter(|handle| handle.match_color_result(&other, &color_result))
            .for_each(|handle| {
                println!("{}", Handle::handle_to_string(&handle));
            });
    }
}
