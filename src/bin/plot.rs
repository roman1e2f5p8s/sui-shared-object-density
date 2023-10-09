use std::fs;
use memmap;
use clap::Parser;
use std::io::Write;
use std::path::Path;
use std::collections::BTreeMap;
use serde_json;

use shared_object_density::args::plot::*;
use shared_object_density::types::*;

fn main() {
    let args = Args::parse();

    let results_dir = Path::new("results");
    let  epoch2checkpoint_file = fs::File::open(results_dir.join("epoch2checkpoint.json")).
        expect("File not found!");

    let epoch2checkpoint_json: BTreeMap<usize, Epoch> = 
        serde_json::from_reader(epoch2checkpoint_file).
        expect("JSON was not properly formatted!");

    let data_dir = Path::new("data");
    let mut data_files: Vec<_> = fs::read_dir(data_dir).
        expect("Couldn't access directory!").
        map(|f| f.unwrap()).
        collect();
    data_files.sort_by_key(|f| f.metadata().unwrap().modified().unwrap());

    let mut epochs_data: BTreeMap<usize, EpochData> = BTreeMap::new();
    let mut epoch: usize = 0;
    epochs_data.insert(epoch, EpochData {
        num_txs_total: 0,
        num_txs_touching_shared_objs: 0,
        density: 0.0,
        num_shared_objects: 0,
        num_checkpoints: 0,
        avg_interval_data: args.intervals.iter().map(|i| (*i, AvgIntervalData{
            contention_degree: 0.0,
            obj_touchability: 0.0,
        })).collect(),
    });

    // auxiliary variables to calculate contention level
    let mut counts_per_interval: BTreeMap<u64, IntervalCounts> = args
        .intervals
        .iter()
        .map(|i| (*i, IntervalCounts {
            num_txs: 0,
            num_obj: 0,
            num_obj_touched_by_more_than_one_tx: 0,
        }))
        .collect();

    for (i, data_file) in data_files.iter().enumerate() {

        print!("\rWorking on file {}/{}...", i + 1, data_files.len());
        let _ = std::io::stdout().flush();

        let file = fs::File::open(data_file.path())
            .expect("File not found!");
        let mmap = unsafe {memmap::Mmap::map(&file)}.unwrap();
        let content = std::str::from_utf8(&mmap).unwrap();
        let json: ResultData = serde_json::from_str(content).unwrap();

        // let json: ResultData = 
        //     serde_json::from_reader(file).
        //     expect("JSON was not properly formatted!");
        
        for (checkpoint, checkpoint_data) in json.checkpoints.into_iter() {
            if checkpoint > epoch2checkpoint_json.
                    get(&epoch).unwrap().end_checkpoint.try_into().unwrap() {
                // The epoch ends: calculate metrics per epoch

                // Calculate density as the ratio of the number of TXs touching
                // shared objects to the total number of TXs per epoch
                epochs_data.get_mut(&epoch).unwrap().density = 
                    epochs_data.get(&epoch).unwrap().num_txs_touching_shared_objs as f64 /
                    epochs_data.get(&epoch).unwrap().num_txs_total as f64;

                for interval in &args.intervals {
                    // Calculate contention degree as the sum of contention degrees
                    // for all intervals within that epoch divided by the number of intervals
                    epochs_data
                        .get_mut(&epoch)
                        .unwrap()
                        .avg_interval_data
                        .get_mut(&interval)
                        .unwrap() 
                        .contention_degree
                            /= epochs_data.get(&epoch).unwrap().num_checkpoints as f64 / *interval as f64;

                    // Calculate object touchebility as the sum of object touchabilities
                    // for all intervals within that epoch divided by the number of intervals
                    epochs_data
                        .get_mut(&epoch)
                        .unwrap()
                        .avg_interval_data
                        .get_mut(&interval)
                        .unwrap() 
                        .obj_touchability
                            /= epochs_data.get(&epoch).unwrap().num_checkpoints as f64 / *interval as f64;

                    // update contention degree counters at the end of epoch
                    counts_per_interval
                        .get_mut(&interval)
                        .unwrap()
                        .num_txs = 0;
                    counts_per_interval
                        .get_mut(&interval)
                        .unwrap()
                        .num_obj = 0;
                    counts_per_interval
                        .get_mut(&interval)
                        .unwrap()
                        .num_obj_touched_by_more_than_one_tx = 0;
                }

                // proceed to the next epoch
                epoch += 1;
                epochs_data.insert(epoch, EpochData {
                    num_txs_total: 0,
                    num_txs_touching_shared_objs: 0,
                    density: 0.0,
                    num_shared_objects: 0,
                    num_checkpoints: 0,
                    avg_interval_data: args.intervals.iter().map(|i| (*i, AvgIntervalData{
                        contention_degree: 0.0,
                        obj_touchability: 0.0,
                    })).collect(),
                });
            }

            // Update the total number of TXs
            epochs_data.get_mut(&epoch).unwrap().num_txs_total += 
                checkpoint_data.num_txs_total;

            // Update the number of TXs touching shared objects
            epochs_data.get_mut(&epoch).unwrap().num_txs_touching_shared_objs += 
                checkpoint_data.num_txs_touching_shared_objs;

            // Update the number of checkpoints
            epochs_data.get_mut(&epoch).unwrap().num_checkpoints += 1;

            // Update the number of shared objects
            epochs_data.get_mut(&epoch).unwrap().num_shared_objects += checkpoint_data.shared_objects.len();

            for (_, tx_list) in checkpoint_data.shared_objects.into_iter() {
                for interval in &args.intervals {
                    counts_per_interval
                        .get_mut(&interval)
                        .unwrap()
                        .num_txs += tx_list.len() as u64;
                    counts_per_interval
                        .get_mut(&interval)
                        .unwrap()
                        .num_obj += 1;
                    if tx_list.len() > 1 {
                        counts_per_interval
                            .get_mut(&interval)
                            .unwrap()
                            .num_obj_touched_by_more_than_one_tx += 1;
                    }
                }
            }

            for interval in &args.intervals {
                // do this every `interval` checkpoints
                if (checkpoint + 1) % interval == 0 {
                    // Calculate contention degree as the number of TXs touching shared
                    // objects divided by the number of touched shared objects
                    let x: f64 = counts_per_interval.get(&interval).unwrap().num_txs as f64 / 
                        counts_per_interval.get(&interval).unwrap().num_obj as f64;

                    if !x.is_nan() {
                        // Sum up contention degree
                        epochs_data
                            .get_mut(&epoch)
                            .unwrap()
                            .avg_interval_data
                            .get_mut(&interval)
                            .unwrap() 
                            .contention_degree
                                += x;
                    }

                    // Calculate object touchability as the number of objects touched by
                    // more than one TX divided by the number of shared objects
                    let y: f64 = counts_per_interval.get(&interval).unwrap()
                        .num_obj_touched_by_more_than_one_tx as f64 / 
                        counts_per_interval.get(&interval).unwrap().num_obj as f64;

                    if !y.is_nan() {
                        // Sum up contention degree
                        epochs_data
                            .get_mut(&epoch)
                            .unwrap()
                            .avg_interval_data
                            .get_mut(&interval)
                            .unwrap() 
                            .obj_touchability
                                += y;
                    }

                    // renew counters needed for contention degree
                    counts_per_interval
                        .get_mut(&interval)
                        .unwrap()
                        .num_txs = 0;
                    counts_per_interval
                        .get_mut(&interval)
                        .unwrap()
                        .num_obj = 0;
                    counts_per_interval
                        .get_mut(&interval)
                        .unwrap()
                        .num_obj_touched_by_more_than_one_tx = 0;
                }
            }
        }
    }
    println!();

    let _ = fs::write(results_dir.join("plotme.json"), serde_json::to_string_pretty(&epochs_data).
            unwrap());
    println!("{:#?}", epochs_data);
}
