use clap::Parser;
use log::info;
use prover::{
    utils::{get_block_trace_from_file, init_env_and_log, load_or_download_params},
    zkevm::{circuit::AGG_DEGREE, Prover},
};
use std::{fs, path::PathBuf, time::Instant};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Get params and write into file.
    #[clap(short, long = "params", default_value = "prover/test_params")]
    params_path: String,
    /// Get BlockTrace from file or dir.
    #[clap(
        short,
        long = "trace",
        default_value = "prover/tests/traces/empty.json"
    )]
    trace_path: String,
}

fn main() {
    init_env_and_log("prove");
    std::env::set_var("VERIFY_CONFIG", "./prover/configs/verify_circuit.config");

    let args = Args::parse();
    let agg_params = load_or_download_params(&args.params_path, *AGG_DEGREE)
        .expect("failed to load or create params");

    // 通过设置参数初始化prover
    let mut prover = Prover::from_params(agg_params);

    // 从文件中加载轨迹信息
    let mut traces = Vec::new();
    let trace_path = PathBuf::from(&args.trace_path);
    if trace_path.is_dir() {
        for entry in fs::read_dir(trace_path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() && path.to_str().unwrap().ends_with(".json") {
                let block_trace = get_block_trace_from_file(path.to_str().unwrap());
                traces.push(block_trace);
            }
        }
    } else {
        let block_trace = get_block_trace_from_file(trace_path.to_str().unwrap());
        traces.push(block_trace);
    }

    let mut proof_dir = PathBuf::from("proof_data");

    // 调用prover.gen_chunk_proof生成chunk proof
    let now = Instant::now();
    let chunk_proof = prover
        .gen_chunk_proof(traces.as_slice())
        .expect("cannot generate chunk proof");
    info!(
        "finish generating chunk proof, elapsed: {:?}",
        now.elapsed()
    );

    fs::create_dir_all(&proof_dir).unwrap();
    chunk_proof.dump(&mut proof_dir, "chunk").unwrap();
}
