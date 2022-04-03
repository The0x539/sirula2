extern crate gtk4 as gtk;

mod app;
mod config;
mod history;
mod launcher_entry;
mod model;
mod util;

fn main() {
    let model = model::AppModel::new();
    let app = relm4::RelmApp::new(model);
    app.run();
}
