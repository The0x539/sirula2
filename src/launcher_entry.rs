use gtk::gio::AppInfo;
use gtk::glib::shell_unquote;
use gtk::pango;
use gtk::prelude::*;

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use itertools::Itertools;
use relm4::view;

use super::config::{Config, Field};
use super::history::History;

pub(crate) struct LauncherEntry {
    pub display_string: String,
    pub search_string: String,
    pub extra_range: Option<(usize, usize)>,
    pub info: AppInfo,
    pub score: i64,
    pub last_used: u64,
    pub image: gtk::Image,
    pub label: gtk::Label,
}

impl LauncherEntry {
    fn set_markup(&self, config: &Config) {
        let attr_list = pango::AttrList::new();

        add_attrs(
            &attr_list,
            &config.markup_default,
            0,
            self.display_string.len(),
        );
        if let Some((lo, hi)) = self.extra_range {
            add_attrs(&attr_list, &config.markup_extra, lo, hi);
        }
    }

    pub fn load_all<'a>(
        config: &'a Config,
        history: &'a History,
    ) -> impl Iterator<Item = LauncherEntry> + 'a {
        let icon_theme = gtk::IconTheme::default();
        AppInfo::all()
            .into_iter()
            .filter_map(move |app| load_entry(config, history, &icon_theme, app))
    }

    pub fn update_match(&mut self, pattern: &str, matcher: &SkimMatcherV2, config: &Config) {
        self.set_markup(config);

        let attr_list = self.label.attributes().unwrap_or_default();
        if pattern.is_empty() {
            self.label.set_attributes(None);
            self.score = 100;
        } else if let Some((score, indices)) = matcher.fuzzy_indices(&self.search_string, pattern) {
            for i in indices {
                if i < self.display_string.len() {
                    add_attrs(&attr_list, &config.markup_highlight, i, i + 1);
                }
            }
            self.score = score;
        } else {
            self.score = 0;
        }

        self.label.set_attributes(Some(&attr_list));
    }

    pub fn hide(&mut self) {
        self.score = 0;
    }
}

fn load_entry(
    config: &Config,
    history: &History,
    icon_theme: &gtk::IconTheme,
    app: AppInfo,
) -> Option<LauncherEntry> {
    if !app.should_show() {
        return None;
    }

    let id = app.id()?.to_string();

    // TODO: check exclusions

    let display_string;
    let mut extra_range = None;

    match get_app_field(&app, Field::Id).and_then(|id| config.name_overrides.get(&id)) {
        Some(name) => {
            if let Some(i) = name.find('\r') {
                display_string = name.replace('\r', " ");
                extra_range = Some((i + 1, name.len()));
            } else {
                display_string = name.clone();
            }
        }
        None => {
            let extra = config
                .extra_field
                .first()
                .and_then(|f| get_app_field(&app, *f));

            let name = app.display_name().to_string();
            match extra {
                Some(e)
                    if !(config.hide_extra_if_contained
                        && name.to_lowercase().contains(&e.to_lowercase())) =>
                {
                    display_string = format!("{name} {e}");
                    let extra_start = name.len() + 1;
                    extra_range = Some((extra_start, extra_start + e.len()));
                }
                _ => display_string = name,
            }
        }
    };

    let hidden = config
        .hidden_fields
        .iter()
        .filter_map(|field| get_app_field(&app, *field))
        .join(" ");

    let search_string = if hidden.is_empty() {
        display_string.clone()
    } else {
        format!("{display_string} {hidden}")
    };

    let image = gtk::Image::builder().pixel_size(config.icon_size).build();
    if let Some(icon) = app.icon() {
        // TODO: replicate sirula behavior
        if true || icon_theme.has_gicon(&icon) {
            image.set_from_gicon(&icon);
        }
    }
    image.style_context().add_class("app-icon");

    view! {
        label = gtk::Label {
            set_xalign: 0.0,
            set_label: &display_string,
            set_wrap: true,
            set_ellipsize: pango::EllipsizeMode::End,
            set_lines: config.lines,
        }
    }

    let last_used = if config.recent_first {
        history.last_used.get(&id).copied().unwrap_or(0)
    } else {
        0
    };

    let entry = LauncherEntry {
        display_string,
        search_string,
        extra_range,
        info: app,
        score: 100,
        last_used,
        image,
        label,
    };
    entry.set_markup(config);
    Some(entry)
}

fn get_app_field(app: &AppInfo, field: Field) -> Option<String> {
    let s = match field {
        Field::Comment => app.description()?.to_string(),
        Field::Id => app.id()?.as_str().strip_suffix(".desktop")?.to_string(),
        Field::IdSuffix => app.id()?.as_str().rsplit('.').nth(1)?.to_string(),
        Field::Executable => app
            .executable()
            .file_name()
            .and_then(|s| shell_unquote(s).ok())?
            .to_string_lossy()
            .to_string(),
        // TODO: clean up command line from %
        Field::Commandline => app.commandline()?.to_string_lossy().to_string(),
    };
    Some(s)
}

fn add_attrs(list: &pango::AttrList, attrs: &[pango::Attribute], start: usize, end: usize) {
    for attr in attrs {
        let mut attr = attr.clone();
        attr.set_start_index(start as u32);
        attr.set_end_index(end as u32);
        list.insert(attr);
    }
}
