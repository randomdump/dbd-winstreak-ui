// Prevent console window in release builds on Windows. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde::{Deserialize, Serialize};
use slint::{Image, VecModel};
use std::{
    cell::RefCell,
    error::Error,
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
    rc::Rc,
};
slint::include_modules!();

/// Default streak categories—used when streaks.txt doesn't exist or is empty.
static DEFAULT_STREAK_CATEGORIES: &[&str] = &["4k", "3k", "Perkless 4k", "Perkless 3k"];

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StreakCategory {
    name: String,
    current: i32,
    best: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Killer {
    name: String,
    image_path: String,
    streaks: Vec<StreakCategory>,
}

/// Load streak categories from streaks.txt, falling back to defaults if needed.
fn load_streak_categories() -> Vec<String> {
    const STREAKS_FILE: &str = "streaks.txt";

    if let Ok(file) = std::fs::File::open(STREAKS_FILE) {
        let reader = BufReader::new(file);
        let mut categories = Vec::new();

        for line in reader.lines().map_while(Result::ok) {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                categories.push(trimmed.to_string());
            }
        }

        if !categories.is_empty() {
            return categories;
        }
    }

    // File doesn't exist or is empty, create it with defaults
    if let Err(e) = create_default_streaks_file(STREAKS_FILE) {
        eprintln!("Warning: Could not create {}: {}", STREAKS_FILE, e);
    }

    DEFAULT_STREAK_CATEGORIES
        .iter()
        .map(|&s| s.to_string())
        .collect()
}

/// Create a default streaks.txt file with comments explaining how to use it.
fn create_default_streaks_file(path: &str) -> Result<(), Box<dyn Error>> {
    let mut file = std::fs::File::create(path)?;

    writeln!(file, "# Streak Categories Configuration")?;
    writeln!(
        file,
        "# Each line represents a streak type you want to track."
    )?;
    writeln!(
        file,
        "# Lines starting with # are comments and will be ignored."
    )?;
    writeln!(file, "# Empty lines are also ignored.")?;
    writeln!(file, "#")?;
    writeln!(
        file,
        "# Add your own streak types by adding new lines below."
    )?;
    writeln!(file, "# Example: No Add-ons 4k")?;
    writeln!(file, "# Example: Adept Achievement")?;
    writeln!(
        file,
        "# Once you add your streak type, re-run the application."
    )?;
    writeln!(file, "#")?;
    writeln!(file, "# Default streak categories:")?;

    for &category in DEFAULT_STREAK_CATEGORIES {
        writeln!(file, "{}", category)?;
    }

    Ok(())
}

/// Adds any missing categories to each killer and returns whether mutations occurred.
fn ensure_categories(killers: &mut [Killer], categories: &[String]) -> bool {
    let mut changed = false;
    for kr in killers.iter_mut() {
        for cat in categories {
            if !kr.streaks.iter().any(|s| s.name == *cat) {
                kr.streaks.push(StreakCategory {
                    name: cat.clone(),
                    current: 0,
                    best: 0,
                });
                changed = true;
            }
        }
    }
    changed
}

fn format_name(stem: &str) -> String {
    stem.replace('_', " ")
        .chars()
        .fold(String::new(), |mut acc, c| {
            if c.is_uppercase() && acc.chars().last().is_some_and(|p| p.is_lowercase()) {
                acc.push(' ');
            }
            acc.push(c);
            acc
        })
}

fn load_data() -> Vec<Killer> {
    const JSON: &str = "killers.json";
    let mut data_changed = false;
    let streak_categories = load_streak_categories();

    // Try reading existing data
    let mut killers: Vec<Killer> = if let Ok(file) = OpenOptions::new().read(true).open(JSON) {
        let reader = BufReader::new(&file);
        serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };

    // Discover new killers in media directory
    if let Ok(entries) = fs::read_dir("media") {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
            {
                let stem = path.file_stem().unwrap().to_str().unwrap();
                let name = format_name(stem);
                if !killers.iter().any(|k| k.name == name) {
                    killers.push(Killer {
                        name,
                        image_path: path.to_string_lossy().into(),
                        streaks: streak_categories
                            .iter()
                            .map(|n| StreakCategory {
                                name: n.clone(),
                                current: 0,
                                best: 0,
                            })
                            .collect(),
                    });
                    data_changed = true;
                }
            }
        }
    }

    // Ensure all categories from streaks.txt are present
    if ensure_categories(&mut killers, &streak_categories) {
        data_changed = true;
    }

    if data_changed {
        save_data(&killers).ok();
    }

    killers.sort_by(|a, b| a.name.cmp(&b.name));
    killers
}

fn save_data(killers: &[Killer]) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("killers.json")?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, killers)?;
    Ok(())
}

fn update_ui(ui: &AppWindow, killer: &Killer, idx: usize) {
    ui.set_killer_name(killer.name.clone().into());
    let img = Image::load_from_path(Path::new(&killer.image_path)).unwrap_or_default();
    ui.set_killer_image(img);
    let names: Vec<_> = killer
        .streaks
        .iter()
        .map(|s| s.name.clone().into())
        .collect();
    ui.set_streak_category_names(Rc::new(VecModel::from(names)).into());
    let i = idx.min(killer.streaks.len().saturating_sub(1));
    let cat = &killer.streaks[i];
    ui.set_counter(cat.current);
    ui.set_pbValue(cat.best);
    ui.set_selected_streak_category_index(i as i32);
}

fn main() -> Result<(), Box<dyn Error>> {
    let killers = Rc::new(RefCell::new(load_data()));
    let cur_k = Rc::new(RefCell::new(0));
    let cur_s = Rc::new(RefCell::new(0));
    let ui = AppWindow::new()?;

    if let Some(k) = killers.borrow().first() {
        update_ui(&ui, k, 0);
        let names: Vec<_> = killers
            .borrow()
            .iter()
            .map(|k| k.name.clone().into())
            .collect();
        ui.set_killer_names(Rc::new(VecModel::from(names)).into());
        ui.set_selected_killer_index(0);
    }

    // Killer selection handler
    ui.on_killer_selected({
        let ui_weak = ui.as_weak();
        let killers = killers.clone();
        let cur_k = cur_k.clone();
        let cur_s = cur_s.clone();
        move |name| {
            if let Some(ui) = ui_weak.upgrade() {
                if let Some(idx) = killers
                    .borrow()
                    .iter()
                    .position(|k| k.name == name.as_str())
                {
                    *cur_k.borrow_mut() = idx;
                    *cur_s.borrow_mut() = 0;
                    update_ui(&ui, &killers.borrow()[idx], 0);
                    ui.set_selected_killer_index(idx as i32);
                }
            }
        }
    });

    // Streak category handler
    ui.on_streak_category_selected({
        let ui_weak = ui.as_weak();
        let killers = killers.clone();
        let cur_k = cur_k.clone();
        let cur_s = cur_s.clone();
        move |cat| {
            if let Some(ui) = ui_weak.upgrade() {
                let kidx = *cur_k.borrow();
                if let Some(pos) = killers.borrow()[kidx]
                    .streaks
                    .iter()
                    .position(|s| s.name == cat.as_str())
                {
                    *cur_s.borrow_mut() = pos;
                    update_ui(&ui, &killers.borrow()[kidx], pos);
                }
            }
        }
    });

    // Win/Loss recorder
    let ui_weak = ui.as_weak();
    let killers_ref = killers.clone();
    let cur_k_ref = cur_k.clone();
    let cur_s_ref = cur_s.clone();
    let record = move |is_win: bool| {
        if let Ok(mut list) = killers_ref.try_borrow_mut() {
            let kidx = *cur_k_ref.borrow();
            let sidx = *cur_s_ref.borrow();
            if let Some(cat) = list[kidx].streaks.get_mut(sidx) {
                if is_win {
                    cat.current += 1;
                    cat.best = cat.best.max(cat.current);
                } else {
                    cat.current = 0;
                }
                let (current, best) = (cat.current, cat.best);
                drop(list);
                save_data(&killers_ref.borrow()).ok();
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_counter(current);
                    ui.set_pbValue(best);
                }
            }
        }
    };

    ui.on_record_win({
        let r = record.clone();
        move || r(true)
    });

    ui.on_record_loss({
        let r = record.clone();
        move || r(false)
    });

    ui.run()?;
    Ok(())
}
