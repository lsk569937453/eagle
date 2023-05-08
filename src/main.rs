use std::time::Duration;
#[macro_use]
extern crate lazy_static;
use anyhow::anyhow;
use chrono::Timelike;
use clap::Parser;
use std::thread::sleep;
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};
mod base_data;
use base_data::process_data::ProcessData;
use byte_unit::Byte;
use plotters::prelude::*;
use std::sync::RwLock;
use std::thread;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    pid: u32,
}
lazy_static! {
    pub static ref LIST: RwLock<Vec<ProcessData>> = RwLock::new(vec![]);
}
fn main() -> Result<(), anyhow::Error> {
    thread::spawn(move || {
        let res = ctrlc::set_handler(move || {
            if let Err(e) = draw() {
                println!("{:?}", e);
            }
            std::process::exit(0);
        });
        if let Err(e) = res {
            println!("{:?}", e);
        }
    });

    let args: Args = Args::parse();
    start(args.pid)?;
    Ok(())
}
pub fn start(pid_u32: u32) -> Result<(), anyhow::Error> {
    let mut sys = System::new_all();
    let pid = Pid::from_u32(pid_u32);
    loop {
        sys.refresh_all();
        if let Some(process) = sys.process(pid) {
            let cpu_usage = process.cpu_usage();
            let mem_usage = process.memory();
            LIST.write()
                .map_err(|e| anyhow!("{:?}", e))?
                .push(ProcessData::new(cpu_usage, mem_usage));
            sleep(Duration::from_millis(200));
        } else {
            break;
        }
    }
    if LIST.read().map_err(|e| anyhow!("{:?}", e))?.is_empty() {
        println!(
            "Can not get any data from pid {},please change another pid and try again!",
            pid
        );
    } else {
        println!("Finish processing the process {},and start drawing!", pid);
        if let Err(e) = draw() {
            println!("{:?}", e);
        }
    }
    Ok(())
}
pub fn draw() -> Result<(), anyhow::Error> {
    let current_data = LIST.read().map_err(|e| anyhow!("{:?}", e))?.clone();
    if let Err(e) = draw_cpu(current_data.clone()) {
        println!("{:?}", e);
    }
    if let Err(e) = draw_memory(current_data) {
        println!("{:?}", e);
    }
    let current_dir = std::env::current_dir()?;
    println!("The file has been saved in the directory {:?}", current_dir);
    Ok(())
}
pub fn draw_cpu(list: Vec<ProcessData>) -> Result<(), anyhow::Error> {
    let root = BitMapBackend::new("cpu.png", (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;
    let x_start = list[0].current_time;
    let x_end = list[list.len() - 1].current_time;

    let y_min = list
        .iter()
        .map(|x| x.cpu_usage)
        .fold(f32::INFINITY, |a, b| a.min(b));
    let y_max = list
        .iter()
        .map(|x| x.cpu_usage)
        .max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
        .unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("Cpu usage", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_start..x_end, 0f32..(y_max + y_max / 10.0))?;

    chart
        .configure_mesh()
        .x_label_formatter(&|x| format!("{:02}:{:02}:{:02}", x.hour(), x.minute(), x.second()))
        .draw()?;
    chart.draw_series(LineSeries::new(
        list.iter().map(|x| (x.current_time, x.cpu_usage)),
        &RED,
    ))?;

    root.present()?;
    Ok(())
}
pub fn draw_memory(list: Vec<ProcessData>) -> Result<(), anyhow::Error> {
    let root = BitMapBackend::new("memory.png", (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;
    let x_start = list[0].current_time;
    let x_end = list[list.len() - 1].current_time;

    let y_min = list
        .iter()
        .map(|x| x.mem_usage)
        .min()
        .ok_or(anyhow!("Get min error!"))?;
    let y_max = list
        .iter()
        .map(|x| x.mem_usage)
        .max()
        .ok_or(anyhow!("Get max error!"))?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Memory usage", ("sans-serif", 50).into_font())
        .margin(5)
        .margin_left(30)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_start..x_end, 0u64..(y_max + y_max / 10))?;

    chart
        .configure_mesh()
        .x_label_formatter(&|x| format!("{:02}:{:02}:{:02}", x.hour(), x.minute(), x.second()))
        .y_label_formatter(&|y| {
            let byte = Byte::from_bytes(*y as u128);
            byte.get_appropriate_unit(false).to_string()
        })
        .draw()?;
    chart.draw_series(LineSeries::new(
        list.iter().map(|x| (x.current_time, x.mem_usage)),
        &RED,
    ))?;

    root.present()?;
    Ok(())
}
