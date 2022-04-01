use std::cell::RefCell;
use std::rc::Rc;

extern crate gtk4 as gtk;
use gtk::prelude::*;

use fuzzy_matcher::skim::SkimMatcherV2;
use relm4::factory::{FactoryPrototype, FactoryVec};
use relm4::{send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};

mod config;
mod history;
mod launcher_entry;
mod util;

use config::Config;
use launcher_entry::LauncherEntry;

struct AppModel {
    matcher: SkimMatcherV2,
    entries: FactoryVec<LauncherEntry>,
    entry_scores: Rc<RefCell<Vec<i64>>>,
}

impl AppModel {
    fn update_entry_scores(&self) {
        *self.entry_scores.borrow_mut() = self.entries.iter().map(|e| e.score).collect();
    }
}

impl Default for AppModel {
    fn default() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
            entries: FactoryVec::new(),
            entry_scores: Default::default(),
        }
    }
}

enum AppMsg {
    Refresh,
    Selected(i32),
    UpdateCmd,
    UpdateApp(String),
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Refresh => {
                let config = Default::default();
                let history = Default::default();
                self.entries.clear();
                for entry in LauncherEntry::load_all(&config, &history) {
                    self.entries.push(entry);
                }
                self.update_entry_scores();
            }
            AppMsg::Selected(i) => println!("selected {i}"),
            AppMsg::UpdateCmd => {
                for i in 0..self.entries.len() {
                    self.entries.get_mut(i).unwrap().hide();
                }
                self.update_entry_scores();
            }
            AppMsg::UpdateApp(text) => {
                for i in 0..self.entries.len() {
                    self.entries.get_mut(i).unwrap().update_match(
                        &text,
                        &self.matcher,
                        Config::get(),
                    );
                }
                self.update_entry_scores();
            }
        }
        true
    }
}

#[relm4::factory_prototype]
impl FactoryPrototype for LauncherEntry {
    type Factory = FactoryVec<Self>;
    type Widgets = FactoryWidgets;
    type View = gtk::ListBox;
    type Msg = AppMsg;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            append: &self.image,
            append: &self.label,
        }
    }

    fn position(&self, _index: &usize) {}
}

#[relm4::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        gtk::ApplicationWindow {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,
            connect_show(sender) => move |_| {
                send!(sender, AppMsg::Refresh);
            },
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_homogeneous: false,
                append = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_child: listbox = Some(&gtk::ListBox) {
                        set_vexpand: true,
                        set_margin_all: 5,
                        connect_row_activated(sender) => move |_, row| {
                            send!(sender, AppMsg::Selected(row.index()));
                        },
                        // WHY DOES THIS MAKE THE APP HANG
                        set_filter_func: {
                            let scores = model.entry_scores.clone();
                            move |row| {
                                let scores = scores.borrow();
                                scores[row.index() as usize] > 0
                            }
                        },
                        factory!(model.entries),
                    }
                },
                prepend = &gtk::Entry {
                    connect_changed(sender, listbox) => move |entry| {
                        let text = entry.text();
                        let msg = if util::is_cmd(&text, &Config::get().command_prefix) {
                            AppMsg::UpdateCmd
                        } else {
                            AppMsg::UpdateApp(text.to_string())
                        };
                        send!(sender, msg);
                        listbox.invalidate_filter();
                        listbox.select_row(listbox.row_at_index(0).as_ref());
                    },
                },
            }
        }
    }
}

fn main() {
    let model = AppModel::default();
    let app = RelmApp::new(model);
    app.run();
}
