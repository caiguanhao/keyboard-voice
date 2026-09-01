mod app;
mod audio;
mod keymap;
mod settings;
mod syskeys;

fn main() -> eframe::Result<()> {
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("Keyboard Voice")
        .with_resizable(true);

    #[cfg(target_os = "linux")]
    {
        viewport = viewport.with_fullscreen(true).with_decorations(false);
    }

    #[cfg(not(target_os = "linux"))]
    {
        viewport = viewport
            .with_inner_size([1200.0, 675.0])
            .with_min_inner_size([720.0, 420.0]);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Keyboard Voice",
        options,
        Box::new(|cc| Ok(Box::new(app::KeyboardApp::new(cc)))),
    )
}
