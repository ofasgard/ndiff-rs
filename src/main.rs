use std::fs;
use std::env;

use ndiff_rs::host::HostDelta;
use nmap_xml_parser::NmapResults;

fn load_scan(path : &str) -> NmapResults {
	let content = fs::read_to_string(path).unwrap(); // TODO
	let results = NmapResults::parse(&content).unwrap(); // TODO
	results
}

fn usage() {
	eprintln!("USAGE: ndiff-rs scan1.xml scan2.xml");
}

fn main() {
	let args : Vec<String> = env::args().collect();
	if args.len() < 3 { return usage(); }
	
	let (left_path, right_path) = (&args[1], &args[2]);
	let (left, right) = (load_scan(left_path), load_scan(right_path));
	
	let deltas = HostDelta::from_scans(&left, &right);
	for delta in &deltas {
		print!("{}", delta.to_string());
	}

}

// TODO: add more complex CLI options using clap
// options: display all (default), display new hosts, display gone hosts, display changed hosts
// should also display timestamps extracted from NmapResults
// also, a JSON output option
