#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

pub fn run_gui() -> eframe::Result {
	let mut options = eframe::NativeOptions::default();
	options.viewport = egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]);

	eframe::run_native(
		"ndiff-rs", // app name
		options, // native options
		Box::new(|_cc| { Ok(Box::<NDiffApp>::default()) }), // closure that creates your app
	)
}

struct NDiffApp {}

impl Default for NDiffApp {
	fn default() -> Self {
		Self {}
	}
}

impl eframe::App for NDiffApp {
	fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
	}
}
