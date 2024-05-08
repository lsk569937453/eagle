use std::time::Duration;
mod database;
use anyhow::anyhow;
use chrono::Timelike;
use clap::Parser;
#[macro_use]
extern crate anyhow;
use sqlx::Sqlite;
use sysinfo::{Pid, System};
use tokio::time::{sleep, Instant, Interval};
mod base_data;
use crate::database::init::create_pool;
use base_data::process_data::ProcessData;
use byte_unit::Byte;
use chrono::DateTime;
use chrono::Local;
use chrono::NaiveDateTime;
use plotters::prelude::*;
use sqlx::Pool;
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
    let pool = create_pool().await?;
    let pool_for_delete = pool.clone(); // Clone the pool for delete_old_data

    tokio::spawn(async move {
        delete_old_data(pool_for_delete).await.unwrap();
    });
    // let args: Args = Args::parse();
    start(pool).await?;
    Ok(())
}
pub async fn delete_old_data(pool: Pool<Sqlite>) -> Result<(), anyhow::Error> {
    let mut interval = interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        let current_time = Local::now();
        let three_days_ago = current_time - Duration::from_secs(24 * 3600 * 5);
        let data_str = three_days_ago.format("%Y-%m-%d %H:%M:%S").to_string();
        sqlx::query("DELETE FROM system_data WHERE timestamp < $1")
            .bind(data_str)
            .execute(&pool)
            .await?;
    }

    Ok(())
}
pub async fn start(pool: Pool<Sqlite>) -> Result<(), anyhow::Error> {
    let mut sys = System::new_all();
    let mut interval = interval(Duration::from_millis(1000));
    let mut networks = Networks::new_with_refreshed_list();

    loop {
        interval.tick().await;
        sys.refresh_all();
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

        // Network interfaces name, total data received and total data transmitted:
        let mut total_received = 0;
        let mut total_upload = 0;
        networks.refresh();
        for (_, data) in &networks {
            total_received += data.received();
            total_upload += data.transmitted();
        }
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
        let now = Instant::now();
        sqlx::query(&sql)
            .execute(&pool)
            .await
            .map_err(|e| anyhow!("insert error, the error is {}", e))?;
    }

    Ok(())
}
