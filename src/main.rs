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

/// Default streak categories for killers.
static DEFAULT_KILLER_STREAK_CATEGORIES: &[&str] = &["4k", "3k", "Perkless 4k", "Perkless 3k"];
/// Default streak categories for survivor.
static DEFAULT_SURVIVOR_STREAK_CATEGORIES: &[&str] = &["Solo escape", "3 out"];

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StreakCategory {
    name: String,
    current: i32,
    best: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Character {
    name: String,
    image_path: String,
    streaks: Vec<StreakCategory>,
}

/// Load streak categories from a text file, falling back to defaults if needed.
fn load_categories_from_file(path: &str, defaults: &[&str]) -> Vec<String> {
    if let Ok(file) = std::fs::File::open(path) {
        let reader = BufReader::new(file);
        let categories: Vec<String> = reader
            .lines()
            .map_while(Result::ok)
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();

        if !categories.is_empty() {
            return categories;
        }
    }

    // File doesn't exist or is empty, create it with defaults
    if let Err(e) = create_default_streaks_file(path, defaults) {
        eprintln!("Warning: Could not create {}: {}", path, e);
    }

    defaults.iter().map(|&s| s.to_string()).collect()
}

/// Create a default streaks text file with comments explaining how to use it.
fn create_default_streaks_file(path: &str, defaults: &[&str]) -> Result<(), Box<dyn Error>> {
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
    writeln!(file, "# Default streak categories:")?;
    for &category in defaults {
        writeln!(file, "{}", category)?;
    }
    Ok(())
}

/// Adds any missing categories to a character and returns whether mutations occurred.
fn ensure_categories(character: &mut Character, categories: &[String]) -> bool {
    let mut changed = false;
    for cat_name in categories {
        if !character.streaks.iter().any(|s| s.name == *cat_name) {
            character.streaks.push(StreakCategory {
                name: cat_name.clone(),
                current: 0,
                best: 0,
            });
            changed = true;
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

fn load_data() -> Vec<Character> {
    const JSON: &str = "streaks.json";
    let mut data_changed = false;

    let killer_cats =
        load_categories_from_file("killer_streaks.txt", DEFAULT_KILLER_STREAK_CATEGORIES);
    let survivor_cats =
        load_categories_from_file("survivor_streaks.txt", DEFAULT_SURVIVOR_STREAK_CATEGORIES);

    let mut characters: Vec<Character> = if let Ok(file) = OpenOptions::new().read(true).open(JSON)
    {
        let reader = BufReader::new(&file);
        serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };

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
                if !characters.iter().any(|c| c.name == name) {
                    let cats_to_use = if name.eq_ignore_ascii_case("survivor") {
                        &survivor_cats
                    } else {
                        &killer_cats
                    };
                    characters.push(Character {
                        name,
                        image_path: path.to_string_lossy().into(),
                        streaks: cats_to_use
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

    let mut categories_updated = false;
    for character in &mut characters {
        let cats_to_use = if character.name.eq_ignore_ascii_case("survivor") {
            &survivor_cats
        } else {
            &killer_cats
        };
        if ensure_categories(character, cats_to_use) {
            categories_updated = true;
        }
    }
    if categories_updated {
        data_changed = true;
    }

    if data_changed {
        save_data(&characters).ok();
    }

    characters.sort_by(|a, b| a.name.cmp(&b.name));
    characters
}

fn save_data(characters: &[Character]) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("streaks.json")?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, characters)?;
    Ok(())
}

fn update_ui(ui: &AppWindow, character: &Character, streak_idx: usize) {
    ui.set_killer_name(character.name.clone().into());
    let img = Image::load_from_path(Path::new(&character.image_path)).unwrap_or_default();
    ui.set_killer_image(img);
    let names: Vec<_> = character
        .streaks
        .iter()
        .map(|s| s.name.clone().into())
        .collect();
    ui.set_streak_category_names(Rc::new(VecModel::from(names)).into());
    let i = streak_idx.min(character.streaks.len().saturating_sub(1));
    if let Some(cat) = character.streaks.get(i) {
        ui.set_counter(cat.current);
        ui.set_pbValue(cat.best);
        ui.set_selected_streak_category_index(i as i32);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let characters = Rc::new(RefCell::new(load_data()));
    let current_char_idx = Rc::new(RefCell::new(0));
    let current_streak_idx = Rc::new(RefCell::new(0));
    let ui = AppWindow::new()?;

    if let Some(c) = characters.borrow().first() {
        update_ui(&ui, c, 0);
        let names: Vec<_> = characters
            .borrow()
            .iter()
            .map(|c| c.name.clone().into())
            .collect();
        ui.set_killer_names(Rc::new(VecModel::from(names)).into());
        ui.set_selected_killer_index(0);
    }

    ui.on_killer_selected({
        let ui_weak = ui.as_weak();
        let characters = characters.clone();
        let current_char_idx = current_char_idx.clone();
        let current_streak_idx = current_streak_idx.clone();
        move |name| {
            if let Some(ui) = ui_weak.upgrade() {
                if let Some(idx) = characters
                    .borrow()
                    .iter()
                    .position(|c| c.name == name.as_str())
                {
                    *current_char_idx.borrow_mut() = idx;
                    *current_streak_idx.borrow_mut() = 0;
                    update_ui(&ui, &characters.borrow()[idx], 0);
                    ui.set_selected_killer_index(idx as i32);
                }
            }
        }
    });

    ui.on_streak_category_selected({
        let ui_weak = ui.as_weak();
        let characters = characters.clone();
        let current_char_idx = current_char_idx.clone();
        let current_streak_idx = current_streak_idx.clone();
        move |cat| {
            if let Some(ui) = ui_weak.upgrade() {
                let char_idx = *current_char_idx.borrow();
                let char_data = characters.borrow();
                if let Some(pos) = char_data[char_idx]
                    .streaks
                    .iter()
                    .position(|s| s.name == cat.as_str())
                {
                    *current_streak_idx.borrow_mut() = pos;
                    let selected_streak_category = &char_data[char_idx].streaks[pos];
                    ui.set_counter(selected_streak_category.current);
                    ui.set_pbValue(selected_streak_category.best);
                    ui.set_selected_streak_category_index(pos as i32);
                }
            }
        }
    });

    let record = {
        let ui_weak = ui.as_weak();
        let characters_ref = characters.clone();
        let current_char_idx_ref = current_char_idx.clone();
        let current_streak_idx_ref = current_streak_idx.clone();
        move |is_win: bool| {
            if let Ok(mut list) = characters_ref.try_borrow_mut() {
                let char_idx = *current_char_idx_ref.borrow();
                let s_idx = *current_streak_idx_ref.borrow();
                let character = &mut list[char_idx];

                if let Some(cat) = character.streaks.get_mut(s_idx) {
                    if is_win {
                        cat.current += 1;
                        cat.best = cat.best.max(cat.current);
                    } else {
                        cat.current = 0;
                    }
                }

                // This killer-specific logic should not run for survivor.
                if is_win && !character.name.eq_ignore_ascii_case("survivor") {
                    if let Some(best_4k) = character
                        .streaks
                        .iter()
                        .find(|s| s.name == "4k")
                        .map(|s| s.best)
                    {
                        if let Some(three_k_streak) =
                            character.streaks.iter_mut().find(|s| s.name == "3k")
                        {
                            three_k_streak.best = three_k_streak.best.max(best_4k);
                        }
                    }
                }

                let (current, best) = if let Some(cat) = character.streaks.get(s_idx) {
                    (cat.current, cat.best)
                } else {
                    (0, 0)
                };

                drop(list);

                save_data(&characters_ref.borrow()).ok();

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
