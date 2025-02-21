use std::fs;
use chrono::DateTime;
use clap::Parser;

use ndiff_rs::host::HostDelta;
use nmap_xml_parser::NmapResults;

#[derive(Parser, Debug)]
#[command(version, about = "A diffing tool for NMap scans in XML format.", long_about = None)]
struct Args {
	left_path: String,
	right_path: String
}

#[derive(Debug)]
pub enum Error {
	FileRead(std::io::Error),
	FileParse(nmap_xml_parser::Error)
}

fn load_scan(path : &str) -> Result<NmapResults,Error> {
	let content = match fs::read_to_string(path) {
		Ok(x) => x,
		Err(e) => return Err(Error::FileRead(e))
	};
	
	let results = match NmapResults::parse(&content) {
		Ok(x) => x,
		Err(e) => return Err(Error::FileParse(e))
	};
	
	Ok(results)
}

fn get_time(scan : &NmapResults) -> String {
	match DateTime::from_timestamp(scan.scan_start_time, 0) {
		Some(x) => format!("{}", x),
		None => "<unknown start time>".to_string()
	}
}

fn main() {
	let args = Args::parse();
	
	let left = match load_scan(&args.left_path) {
		Ok(x) => x,
		Err(e) => { println!("Failed to parse {}: {:?}", args.left_path, e); return; }
	};
	
	let right = match load_scan(&args.right_path) {
		Ok(x) => x,
		Err(e) => { println!("Failed to parse '{}': {:?}", args.right_path, e); return; }
	};
	
	println!("Left Scan: {}", get_time(&left)); 
	println!("Right Scan: {}", get_time(&right));
	println!("");
	
	
	let deltas = HostDelta::from_scans(&left, &right);
	for delta in &deltas {
		print!("{}", delta.to_string());
	}

}

// TODO
// options: display all (default), display new hosts, display gone hosts, display changed hosts
// also, a JSON output option
