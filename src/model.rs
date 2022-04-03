use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;

use fuzzy_matcher::skim::SkimMatcherV2;
use relm4::factory::{Factory, FactoryPrototype};
use relm4::{AppUpdate, Model, Sender};

use crate::app::{AppMsg, AppWidgets};
use crate::config::Config;
use crate::launcher_entry::LauncherEntry;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Update {
    None,
    Invalidate,
    Reload,
}

pub(crate) struct AppModel {
    matcher: SkimMatcherV2,
    entries: Vec<LauncherEntry>,
    scores: Rc<[Cell<i64>]>,
    update: Cell<Update>,
}

impl AppModel {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
            entries: Vec::new(),
            scores: Rc::new([]),
            update: Cell::new(Update::Reload),
        }
    }

    fn hide_entries(&mut self) {
        for (entry, score) in self.entries.iter_mut().zip(&*self.scores) {
            let old_score = entry.score;
            entry.hide();
            let new_score = entry.score;
            if new_score != old_score {
                score.set(new_score);
                self.update.set(self.update.get().max(Update::Invalidate));
            }
        }
    }

    fn update_entries(&mut self, text: &str, config: &Config) {
        for (entry, score) in self.entries.iter_mut().zip(&*self.scores) {
            let old_score = entry.score;
            entry.update_match(text, &self.matcher, config);
            let new_score = entry.score;
            if new_score != old_score {
                score.set(new_score);
                self.update.set(self.update.get().max(Update::Invalidate));
            }
        }
    }

    fn replace_entries(&mut self, entries: Vec<LauncherEntry>) {
        self.scores = entries.iter().map(|entry| Cell::new(entry.score)).collect();
        self.entries = entries;
        self.update.set(self.update.get().max(Update::Reload));
    }
}

impl Factory<LauncherEntry, gtk::ListBox> for AppModel {
    type Key = ();

    fn generate(&self, view: &gtk::ListBox, sender: Sender<AppMsg>) {
        match self.update.replace(Update::None) {
            Update::None => {}
            Update::Invalidate => {
                view.invalidate_filter();
                view.invalidate_sort();
            }
            Update::Reload => {
                view.unset_filter_func();
                view.unset_sort_func();

                view.select_all();
                view.selected_foreach(gtk::ListBox::remove);

                for (i, entry) in self.entries.iter().enumerate() {
                    let widgets = entry.init_view(&i, sender.clone());
                    view.append(LauncherEntry::root_widget(&widgets));
                }

                let scores = self.scores.clone();
                let get_score = move |row: &gtk::ListBoxRow| scores[row.index() as usize].get();
                {
                    let get_score = get_score.clone();
                    view.set_filter_func(move |row| get_score(row) > 0);
                }
                view.set_sort_func(move |a, b| get_score(a).cmp(&get_score(b)).into());
            }
        }
    }
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
                let config = Config::get();
                let history = Default::default();
                self.replace_entries(LauncherEntry::load_all(&config, &history).collect());
            }
            AppMsg::Selected(i) => println!("selected {i}"),
            AppMsg::UpdateCmd => self.hide_entries(),
            AppMsg::UpdateApp(text) => self.update_entries(&text, Config::get()),
        }
        true
    }
}
