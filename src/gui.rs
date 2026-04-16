#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::Layout;
use egui::Align;
use egui::widgets::Separator;

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
			first_scan: Option<String>,
			second_scan: Option<String>
}

impl Default for NDiffApp {
	fn default() -> Self {
		Self {
			first_scan: None,
			second_scan: None
		}
	}
}

impl eframe::App for NDiffApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show_inside(ui, |ui| {
			ui.label("Drag and drop two Nmap XML scans into this window!");
			ui.add(Separator::default().spacing(32.0));
			
			ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
				ui.with_layout(Layout::top_down(Align::TOP), |ui| {
					ui.heading("First Scan");
					ui.add_space(32.0);
					match &self.first_scan {
						Some(scan) => todo!(),
						None => ui.code_editor(&mut "Waiting for a scan...")
					}
				});
				ui.with_layout(Layout::top_down(Align::TOP), |ui| {
					ui.heading("Second Scan");
					ui.add_space(32.0);
					match &self.second_scan {
						Some(scan) => todo!(),
						None => ui.code_editor(&mut "Waiting for a scan...")
					}
				});
			});
		});
	}
}
