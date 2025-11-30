mod process; 
mod user;   
mod manager;
mod gui;

use gui::ProcessManagerApp;

fn main() -> eframe::Result<()> {
    // Configure native options for the GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Linux Process Manager - TLI"),
        ..Default::default()
    };

    // Run the GUI application
    eframe::run_native(
        "Linux Process Manager",
        options,
        Box::new(|cc| Box::new(ProcessManagerApp::new(cc))),
    )
}
