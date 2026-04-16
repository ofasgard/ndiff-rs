#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::fs;
use chrono::DateTime;
use eframe::egui;
use egui::Layout;
use egui::Align;
use egui::widgets::Separator;
use rfd::FileDialog;

use crate::host::HostDelta;
use nmap_xml_parser::NmapResults;

pub fn run_gui() -> eframe::Result {
	let mut options = eframe::NativeOptions::default();
	options.viewport = egui::ViewportBuilder::default().with_inner_size([640.0, 640.0]);

	eframe::run_native(
		"ndiff-rs", // app name
		options, // native options
		Box::new(|_cc| { Ok(Box::<NDiffApp>::default()) }), // closure that creates your app
	)
}

struct NDiffApp {
			left_scan: Option<NmapResults>,
			right_scan: Option<NmapResults>,
			deltas: Vec<HostDelta>,
			processed: bool
}

impl Default for NDiffApp {
	fn default() -> Self {
		Self {
			left_scan: None,
			right_scan: None,
			deltas: Vec::new(),
			processed: false
		}
	}
}

impl eframe::App for NDiffApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show_inside(ui, |ui| {
			ui.label("Load two Nmap XML scans!");
			ui.add(Separator::default().spacing(32.0));
			
			ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
				ui.with_layout(Layout::top_down(Align::TOP), |ui| {
					ui.set_width(320.0);
					ui.heading("First Scan");
					ui.add_space(32.0);
					match &self.left_scan {
						Some(_) => { ui.label(format!("Scan loaded!\nScan Time: {}", get_time(&self.left_scan.clone().unwrap()))); },
						None => { 
							ui.label("Waiting for a scan...");
							if ui.button("Open file...").clicked() && let Some(path) = FileDialog::new().pick_file() {
								let path_str = path.display().to_string();
								self.left_scan = self.load_scan(path_str, ui);
							}
						}
					};
					if self.processed { 
						self.render_deltas(); 
					}
				});
				
				ui.add(Separator::default().spacing(16.0));
				
				ui.with_layout(Layout::top_down(Align::TOP), |ui| {
					ui.set_width(320.0);
					ui.heading("Second Scan");
					ui.add_space(32.0);
					match &self.right_scan {
						Some(_) => { ui.label(format!("Scan loaded!\nScan Time: {}", get_time(&self.right_scan.clone().unwrap()))); },
						None => { 
							ui.label("Waiting for a scan...");
							if ui.button("Open file...").clicked() && let Some(path) = FileDialog::new().pick_file() {
								let path_str = path.display().to_string();
								self.right_scan = self.load_scan(path_str, ui);
							}
						}
					};
					if self.processed {
						self.render_deltas();
					}
				});
			});
			
			if self.left_scan.is_some() && self.right_scan.is_some() && !self.processed {
				self.deltas = HostDelta::from_scans(&self.left_scan.clone().unwrap(), &self.right_scan.clone().unwrap());
				self.processed = true;
			}
		});
	}
}

impl NDiffApp {
	fn load_scan(&mut self, path: String, _ui: &mut egui::Ui) -> Option<NmapResults> {
		let content = match fs::read_to_string(path) {
			Ok(x) => x,
			Err(_) => {
				return None; // TODO display some error
			}
		};
		
		match NmapResults::parse(&content) {
			Ok(x) => Some(x),
			Err(_) => {
				None // TODO display some error
			}
		}
	}
	
	fn render_deltas(&mut self) {
		// TODO
	}
}

fn get_time(scan : &NmapResults) -> String {
	match DateTime::from_timestamp(scan.scan_start_time, 0) {
		Some(x) => format!("{}", x),
		None => "<unknown start time>".to_string()
	}
}
