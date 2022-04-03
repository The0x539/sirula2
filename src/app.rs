use gtk::prelude::*;

use relm4::factory::{FactoryPrototype, FactoryVec};
use relm4::{send, WidgetPlus, Widgets};

use crate::config::Config;
use crate::launcher_entry::LauncherEntry;
use crate::model::AppModel;

pub enum AppMsg {
    Refresh,
    Selected(i32),
    UpdateCmd,
    UpdateApp(String),
}

#[relm4::factory_prototype(pub)]
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

#[relm4::widget(pub)]
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
                        factory!(*model),
                    }
                },
                prepend = &gtk::Entry {
                    connect_changed(sender) => move |entry| {
                        let text = entry.text();
                        let msg = if crate::util::is_cmd(&text, &Config::get().command_prefix) {
                            AppMsg::UpdateCmd
                        } else {
                            AppMsg::UpdateApp(text.to_string())
                        };
                        send!(sender, msg);
                    },
                },
            }
        }
    }
}
