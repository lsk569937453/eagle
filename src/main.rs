use std::time::Duration;
mod database;
use anyhow::anyhow;
use chrono::Timelike;
use clap::Parser;
#[macro_use]
extern crate anyhow;
use sysinfo::{Pid, System};
use tokio::time::{sleep, Interval};
mod base_data;
use crate::database::init::create_pool;
use base_data::process_data::ProcessData;
use byte_unit::Byte;
use plotters::prelude::*;
use sysinfo::Disks;

use std::thread;
use sysinfo::Networks;
use tokio::time::interval;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    pid: u32,
}
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // let args: Args = Args::parse();
    start().await?;
    Ok(())
}
pub async fn start() -> Result<(), anyhow::Error> {
    let pool = create_pool().await?;
    let mut sys = System::new_all();
    let mut interval = interval(Duration::from_millis(1000));
    loop {
        interval.tick().await;
        sys.refresh_all();
        println!("=> system:");
        // RAM and swap information:
        println!("total memory: {} bytes", sys.total_memory());
        println!("used memory : {} bytes", sys.used_memory());
        println!("total swap  : {} bytes", sys.total_swap());
        println!("used swap   : {} bytes", sys.used_swap());

        for cpu in sys.cpus() {
            print!("{}% ", cpu.cpu_usage());
        }
        let mut read_bytes = 0;
        let mut total_read_bytes = 0;

        let mut write_bytes = 0;
        let mut total_write_bytes = 0;
        for (pid, process) in sys.processes() {
            let disk_usage = process.disk_usage();
            read_bytes += disk_usage.read_bytes;
            total_read_bytes += disk_usage.total_read_bytes;
            write_bytes += disk_usage.written_bytes;
            total_write_bytes += disk_usage.total_written_bytes;
        }
        println!(
            "disk usage:{},{},{},{}",
            read_bytes, total_read_bytes, write_bytes, total_write_bytes
        );

        // Network interfaces name, total data received and total data transmitted:
        let networks = Networks::new_with_refreshed_list();
        println!("=> networks:");
        for (interface_name, data) in &networks {
            println!(
                "{interface_name}: {} B (down) / {} B (up)",
                data.total_received(),
                data.total_transmitted(),
            );
            // If you want the amount of data received/transmitted since last call
            // to `Networks::refresh`, use `received`/`transmitted`.
        }
    }

    Ok(())
}
