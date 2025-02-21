use std::fs;
use std::path::Path;

use ndiff_rs::host::HostDelta;
use nmap_xml_parser::NmapResults;

fn load_scans(folder_str : &str) -> Vec<NmapResults> {
	// Enumerate files in scan folder.
	let folder_path = Path::new(folder_str);
	let paths = fs::read_dir(folder_path).unwrap(); // TODO
	
	// Filter for scan files in *.xml format.
	let mut scan_files : Vec<String> = Vec::new();
	for result in paths {
		if let Ok(x) = result {
			let path = x.path();
			let path_str = path.into_os_string().into_string().unwrap(); // TODO
			if path_str.to_lowercase().ends_with(".xml") { 
				scan_files.push(path_str.to_string());
			}
		}
	}
	
	// Read and parse all scan files.
	let mut output : Vec<NmapResults> = Vec::new();
	for scan_file in scan_files {
		let content = fs::read_to_string(scan_file).unwrap(); // TODO
		let results = NmapResults::parse(&content).unwrap(); // TODO
		output.push(results)
	}
	
	// Sort by timestamp.
	output.sort_by_key(|item| { item.scan_start_time });
	
	output
}

fn main() {
	let scans = load_scans("scans/");
	let deltas = HostDelta::from_scans(&scans[scans.len() - 2], &scans[scans.len() - 1]);
	for delta in &deltas {
		print!("{}", delta.to_string());
	}
}

// CLI provided with 2 files to compare
// options: display all (default), display new hosts, display gone hosts, display changed hosts
// should display timestamps extracted from NmapResults
