use std::fs;
use std::env;

use ndiff_rs::host::HostDelta;
use nmap_xml_parser::NmapResults;

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

fn usage() {
	eprintln!("USAGE: ndiff-rs scan1.xml scan2.xml");
}

fn main() {
	let args : Vec<String> = env::args().collect();
	if args.len() < 3 { return usage(); }
	
	let (left_path, right_path) = (&args[1], &args[2]);
	
	let left = match load_scan(left_path) {
		Ok(x) => x,
		Err(e) => { println!("Failed to parse {}: {:?}", left_path, e); return; }
	};
	
	let right = match load_scan(right_path) {
		Ok(x) => x,
		Err(e) => { println!("Failed to parse '{}': {:?}", right_path, e); return; }
	};
	
	
	let deltas = HostDelta::from_scans(&left, &right);
	for delta in &deltas {
		print!("{}", delta.to_string());
	}

}

// TODO: add more complex CLI options using clap
// options: display all (default), display new hosts, display gone hosts, display changed hosts
// should also display timestamps extracted from NmapResults
// also, a JSON output option
