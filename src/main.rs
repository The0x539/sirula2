extern crate gtk4 as gtk;

mod app;
mod config;
mod history;
mod launcher_entry;
mod util;

fn main() {
    let model = app::AppModel::default();
    let app = relm4::RelmApp::new(model);
    app.run();
}
