#![allow(unused)]
use sscanf::*;
use std::{hint::black_box, time::Instant};

#[cfg(feature = "complex")]
mod complex;
#[cfg(feature = "complex")]
use complex::*;

#[cfg(feature = "complex")]
const INPUT: &str = "BEGIN::version=7::environment=production-eu-west::record[trace=trace-20260905-000042|user=918273645|source=ingestion-worker-primary|attempt=3|payload<primary<config<id=42|name=orders-replicator|enabled=true|retries=5|ratio=0.875> endpoint<host=collector.eu-west.example.internal;port=8443;secure=true;region=eu-west;zone=eu-west-1b;timeout=2500> metrics<method=POST,path=/v1/telemetry/records,status=202,bytes=1048576,latency=1842>>>|checksum=1837465920|finished=true]::signature=sha256-9e107d9c4b7a::END";
#[cfg(not(feature = "complex"))]
const INPUT: &str = "Size: 1920x1080";

#[cfg(not(feature = "multi"))]
pub mod a {
    use crate::*;
    pub fn run_benchmark() {
        #[cfg(not(feature = "complex"))]
        black_box(sscanf!(black_box(INPUT), "Size: {usize}x{usize}").unwrap());
        #[cfg(feature = "complex")]
        black_box(sscanf!(black_box(INPUT), "{BenchmarkMessage}").unwrap());
    }
}

#[cfg(feature = "multi")]
pub mod a {
    use crate::*;
    // #[cfg(not(feature = "new_sscanf"))]
    pub fn run_benchmark() {
        for _ in 0..1000 {
            #[cfg(not(feature = "complex"))]
            black_box(sscanf!(black_box(INPUT), "Size: {usize}x{usize}").unwrap());
            #[cfg(feature = "complex")]
            black_box(sscanf!(black_box(INPUT), "{BenchmarkMessage}").unwrap());
        }
    }
    // #[cfg(feature = "new_sscanf")]
    // pub fn run_benchmark() {
    //     #[cfg(not(feature = "complex"))]
    //     let mut parser = sscanf_parser!("Size: {usize}x{usize}");
    //     #[cfg(feature = "complex")]
    //     let mut parser = sscanf_parser!("{BenchmarkMessage}");
    //     for _ in 0..1000 {
    //         black_box(parser.parse(black_box(INPUT)).unwrap());
    //     }
    // }
}

fn main() {
    a::run_benchmark();
}
