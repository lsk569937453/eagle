use std::time::Duration;
mod database;
use anyhow::anyhow;
use chrono::Timelike;
use clap::Parser;
#[macro_use]
extern crate anyhow;
use sysinfo::{Pid, System};
use tokio::time::{sleep, Instant, Interval};
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
    let mut networks = Networks::new_with_refreshed_list();

    loop {
        interval.tick().await;
        sys.refresh_all();
        println!("=> system:");
        // RAM and swap information:
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let total_swap = sys.total_swap();
        let used_swap = sys.used_swap();

        let mut cpu_uage = 0.0;
        let count = sys.cpus().len();
        for cpu in sys.cpus() {
            cpu_uage += cpu.cpu_usage();
        }
        cpu_uage /= (count * 100) as f32;
        println!("cpu usage:{}", cpu_uage);
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
        let mut total_received = 0;
        let mut total_upload = 0;
        networks.refresh();
        for (_, data) in &networks {
            total_received += data.received();
            total_upload += data.transmitted();
        }
        println!("network usage:{},{}", total_received, total_upload);
        let sql = format!(
            r#"INSERT INTO system_data (total_memory, used_memory, total_swap, used_swap,cpu_usage,disk_usage_read_bytes,disk_usage_written_bytes,network_usage_upload_bytes,network_usage_download_bytes) 
        VALUES ("{}","{}","{}","{}","{}","{}","{}","{}","{}")"#,
            total_memory,
            used_memory,
            total_swap,
            used_swap,
            cpu_uage,
            read_bytes,
            write_bytes,
            total_upload,
            total_received
        );
        println!("sql is {}", sql);
        let now = Instant::now();
        sqlx::query(&sql)
            .execute(&pool)
            .await
            .map_err(|e| anyhow!("insert error, the error is {}", e))?;
        println!("time elasmpsed:{}ms", now.elapsed().as_millis());
    }

    Ok(())
}
