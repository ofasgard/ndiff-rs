#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::fs;
use chrono::DateTime;
use eframe::egui;
use egui::Layout;
use egui::Align;
use egui::widgets::Separator;
use rfd::FileDialog;

use crate::host::HostDiff;
use crate::host::HostDelta;
use crate::host::HostWrapper;
use crate::host::PortsWrapper;
use crate::host::AddressesWrapper;
use crate::host::HostnamesWrapper;
use nmap_xml_parser::NmapResults;
use nmap_xml_parser::host::Host;

pub fn run_gui() -> eframe::Result {
	let mut options = eframe::NativeOptions::default();
	options.viewport = egui::ViewportBuilder::default().with_inner_size([640.0, 640.0]).with_min_inner_size([640.0, 320.0]);

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
		let max_width : f32 = ui.ctx().content_rect().max.x;
	
		egui::CentralPanel::default().show_inside(ui, |ui| {
			ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
				ui.with_layout(Layout::top_down(Align::TOP), |ui| {
					ui.set_width(max_width / 2.0);
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
						self.render_deltas(true, ui); 
					}
				});
				
				ui.add(Separator::default().spacing(16.0));
				
				ui.with_layout(Layout::top_down(Align::TOP), |ui| {
					ui.set_width(max_width / 2.0);
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
						self.render_deltas(false, ui);
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
	
	fn render_deltas(&mut self, left: bool, ui: &mut egui::Ui) {
		for delta in &self.deltas {
			match delta {
				HostDelta::Changed(diff) => self.render_changed(diff, left, ui),
				HostDelta::Unchanged(host) => self.render_unchanged(host, ui),
				HostDelta::Gone(host) => self.render_gone(host, left, ui),
				HostDelta::New(host) => self.render_new(host, left, ui)
			}
		}
	}
	
	fn render_changed(&self, diff: &HostDiff, left: bool, ui: &mut egui::Ui) {
		let report_color = match left {
			true => egui::Color32::from_rgb(0x0, 0x80, 0x0),
			false => egui::Color32::from_rgb(0x80, 0x0, 0x0)
		};
	
		ui.add(Separator::default().spacing(16.0));
		ui.label(egui::RichText::new(format!("{}", diff.title)).underline().color(report_color));
		
		let mut report : String = String::new();
		
		if let Some(status) = &diff.status {
			let current_side = match left { true => status.0.clone(), false => status.1.clone() };
			let status_str = format!("| Status: {} ({})\n", current_side.state.to_string(), current_side.reason);
			report.push_str(&status_str);
		}
		
		if let Some(ports) = &diff.ports {
			let current_side = match left { true => ports.0.clone(), false => ports.1.clone() };
			let wrapped_ports = PortsWrapper(current_side);
			let ports_str = format!("| Ports: {}\n", wrapped_ports.to_string());
			report.push_str(&ports_str);
		}
		
		if let Some(addresses) = &diff.addresses {
			let current_side = match left { true => addresses.0.clone(), false => addresses.1.clone() };
			let wrapped_addresses = AddressesWrapper(current_side);
			let addresses_str = format!("| Addresses: {}\n", wrapped_addresses.to_string());
			report.push_str(&addresses_str);
		}
		
		if let Some(hostnames) = &diff.hostnames {
			let current_side = match left { true => hostnames.0.clone(), false => hostnames.1.clone() };
			let wrapped_hostnames = HostnamesWrapper(current_side);
			let hostnames_str = format!("| Hostnames: {}\n", wrapped_hostnames.to_string());
			report.push_str(&hostnames_str);
		}
		
		ui.label(egui::RichText::new(report).color(report_color));
	}
	
	fn render_unchanged(&self, host: &Host, ui: &mut egui::Ui) {
		let report_color = egui::Color32::from_rgb(0x0, 0x80, 0x0);
		
		let wrapped_host = HostWrapper(host.clone());
		
		ui.add(Separator::default().spacing(16.0));
		ui.label(egui::RichText::new(format!("{}", wrapped_host.get_title())).underline().color(report_color));
		
		let mut report : String = String::new();
		report.push_str("(NO CHANGE)");
		
		ui.label(egui::RichText::new(report).color(report_color));
	}
	
	fn render_gone(&self, host: &Host, left: bool, ui: &mut egui::Ui) {
		let report_color = match left {
			true => egui::Color32::from_rgb(0x0, 0x80, 0x0),
			false => egui::Color32::from_rgb(0x80, 0x0, 0x0)
		};
		
		let wrapped_host = HostWrapper(host.clone());
		
		ui.add(Separator::default().spacing(16.0));
		ui.label(egui::RichText::new(format!("{}", wrapped_host.get_title())).underline().color(report_color));
		
		let mut report : String = String::new();
		
		match left {
			true => report.push_str(&wrapped_host.to_string()),
			false => report.push_str("(HOST GONE)")
		}
		
		ui.label(egui::RichText::new(report).color(report_color));
	}
	
	fn render_new(&self, host: &Host, left: bool, ui: &mut egui::Ui) {
		let report_color = match left {
			true => egui::Color32::from_rgb(0x80, 0x0, 0x0),
			false => egui::Color32::from_rgb(0x0, 0x80, 0x0)
		};
		
		let wrapped_host = HostWrapper(host.clone());
		
		ui.add(Separator::default().spacing(16.0));
		ui.label(egui::RichText::new(format!("{}", wrapped_host.get_title())).underline().color(report_color));
		
		let mut report : String = String::new();
		
		match left {
			true => report.push_str("(NEW HOST)"),
			false => report.push_str(&wrapped_host.to_string())
		}
		
		ui.label(egui::RichText::new(report).color(report_color));
	}
}

fn get_time(scan : &NmapResults) -> String {
	match DateTime::from_timestamp(scan.scan_start_time, 0) {
		Some(x) => format!("{}", x),
		None => "<unknown start time>".to_string()
	}
}
