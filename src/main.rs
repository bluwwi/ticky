#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod config;
mod hooks;
mod packs;

slint::slint! {
    #[edition = "1.19"]

    import "../assets/fonts/Manrope-Variable.ttf";
    import { Palette, VerticalBox } from "std-widgets.slint";

    component Toggle {
        in-out property <bool> on;
        callback toggled();

        width: 44px;
        height: 24px;

        Rectangle {
            width: parent.width;
            height: parent.height;
            border-radius: 12px;
            background: root.on ? #7C5CFF : #262A33;
            animate background { duration: 120ms; }
        }

        Rectangle {
            width: 18px;
            height: 18px;
            y: 3px;
            x: root.on ? parent.width - 21px : 3px;
            border-radius: 9px;
            background: #F4F5F7;
            animate x { duration: 120ms; easing: ease-out; }
        }

        ta := TouchArea {
            clicked => {
                root.on = !root.on;
                root.toggled();
            }
        }
    }

    component PackCard inherits Rectangle {
        in property <string> title;
        in property <string> subtitle;
        in property <bool> selected;
        callback choose();

        height: 78px;
        horizontal-stretch: 1;
        border-radius: 12px;
        background: selected ? #211C3A : #161921;
        border-width: 1px;
        border-color: selected ? #7C5CFF : #232733;

        VerticalLayout {
            padding: 14px;
            spacing: 4px;

            Text {
                text: title;
                color: #F2F3F5;
                font-size: 15px;
                font-weight: 600;
                overflow: elide;
            }

            Text {
                text: subtitle;
                color: #8A90A0;
                font-size: 11px;
                overflow: elide;
            }
        }

        ta := TouchArea {
            clicked => { root.choose(); }
        }
    }

    component ActionButton inherits Rectangle {
        in property <string> label;
        in property <bool> primary: false;
        callback pressed();

        height: 40px;
        min-width: 112px;
        border-radius: 10px;
        background: primary ? (ta.has-hover ? #8B7BFF : #7C5CFF) : (ta.has-hover ? #232833 : #1A1E27);
        border-width: 1px;
        border-color: primary ? #7C5CFF : #2A2F3B;

        Text {
            text: label;
            color: primary ? #FFFFFF : #D7DAE2;
            font-size: 13px;
            font-weight: 600;
            horizontal-alignment: center;
            vertical-alignment: center;
        }

        ta := TouchArea {
            clicked => { root.pressed(); }
        }
    }

    component OptionRow inherits Rectangle {
        in property <string> label;
        in property <string> hint;
        in-out property <bool> on;
        callback toggled();

        height: 56px;
        border-radius: 12px;
        background: #161921;
        border-width: 1px;
        border-color: #232733;

        HorizontalLayout {
            padding-left: 16px;
            padding-right: 14px;
            spacing: 12px;

            VerticalLayout {
                alignment: center;
                horizontal-stretch: 1;
                spacing: 2px;

                Text {
                    text: label;
                    color: #E7E9EE;
                    font-size: 13px;
                    font-weight: 600;
                    vertical-alignment: center;
                }

                Text {
                    text: hint;
                    color: #7E8494;
                    font-size: 11px;
                    overflow: elide;
                }
            }

            Toggle {
                on <=> root.on;
                toggled => { root.toggled(); }
            }
        }
    }

    export component AppWindow inherits Window {
        title: "Ticky";
        default-font-family: "Manrope";
        preferred-width: 560px;
        preferred-height: 636px;
        min-width: 480px;
        min-height: 560px;
        background: #0E0F12;

        in property <[string]> pack-names;
        in property <[string]> pack-authors;
        in-out property <int> current-pack: 0;
        in property <string> current-pack-author: "";
        in property <string> current-pack-license: "";
        in property <string> current-pack-description: "";
        in-out property <bool> sound-enabled: true;
        in-out property <bool> muted: false;
        in-out property <string> status: "Ready";

        callback select-pack(int);
        callback apply();
        callback preview();
        callback reload-packs();
        callback open-data();
        callback show-about();
        callback quit-app();

        init => {
            Palette.color-scheme = ColorScheme.dark;
        }

        MenuBar {
            Menu {
                title: "File";

                MenuItem {
                    title: "Apply settings";
                    shortcut: @keys(Control + S);
                    activated => { root.apply(); }
                }

                MenuSeparator {}

                MenuItem {
                    title: "Open packs folder";
                    activated => { root.open-data(); }
                }

                MenuSeparator {}

                MenuItem {
                    title: "Quit";
                    shortcut: @keys(Control + Q);
                    activated => { root.quit-app(); }
                }
            }

            Menu {
                title: "Sound";

                MenuItem {
                    title: "Preview selected pack";
                    shortcut: @keys(Control + P);
                    activated => { root.preview(); }
                }

                MenuSeparator {}

                MenuItem {
                    title: "Sound enabled";
                    checkable: true;
                    checked <=> root.sound-enabled;
                    activated => { root.apply(); }
                }

                MenuItem {
                    title: "Mute";
                    checkable: true;
                    checked <=> root.muted;
                    activated => { root.apply(); }
                }
            }

            Menu {
                title: "Packs";

                MenuItem {
                    title: "Reload packs";
                    activated => { root.reload-packs(); }
                }

                MenuItem {
                    title: "Preview current pack";
                    activated => { root.preview(); }
                }
            }

            Menu {
                title: "Help";

                MenuItem {
                    title: "About Ticky";
                    activated => { root.show-about(); }
                }
            }
        }

        VerticalBox {
            padding: 22px;
            spacing: 14px;

            VerticalBox {
                spacing: 2px;

                Text {
                    text: "Ticky";
                    color: #F5F6F8;
                    font-size: 28px;
                    font-weight: 700;
                }

                Text {
                    text: "Keyboard and mouse sounds";
                    color: #8A90A0;
                    font-size: 12px;
                }
            }

            Text {
                text: "SOUND PACK";
                color: #6F7585;
                font-size: 11px;
                font-weight: 600;
            }

            HorizontalLayout {
                spacing: 10px;

                for name[i] in root.pack-names : PackCard {
                    title: name;
                    subtitle: i < root.pack-authors.length ? root.pack-authors[i] : "";
                    selected: root.current-pack == i;
                    choose => { root.select-pack(i); }
                }
            }

            Rectangle {
                height: 88px;
                border-radius: 12px;
                background: #161921;
                border-width: 1px;
                border-color: #232733;

                VerticalLayout {
                    padding: 14px;
                    spacing: 6px;

                    Text {
                        text: root.current-pack-description;
                        color: #C9CDD6;
                        font-size: 12px;
                        wrap: word-wrap;
                    }

                    Text {
                        text: "By " + root.current-pack-author + "  ·  " + root.current-pack-license;
                        color: #8A90A0;
                        font-size: 11px;
                        overflow: elide;
                    }
                }
            }

            Text {
                text: "OPTIONS";
                color: #6F7585;
                font-size: 11px;
                font-weight: 600;
            }

            OptionRow {
                label: "Sound enabled";
                hint: "Play sounds for keyboard and mouse input";
                on <=> root.sound-enabled;
                toggled => { root.apply(); }
            }

            OptionRow {
                label: "Mute";
                hint: "Keep the app running without making noise";
                on <=> root.muted;
                toggled => { root.apply(); }
            }

            HorizontalLayout {
                alignment: end;
                spacing: 10px;

                ActionButton {
                    label: "Preview";
                    pressed => { root.preview(); }
                }

                ActionButton {
                    label: "Apply settings";
                    primary: true;
                    pressed => { root.apply(); }
                }
            }

            HorizontalLayout {
                alignment: start;
                spacing: 8px;

                Rectangle {
                    width: 8px;
                    height: 8px;
                    border-radius: 4px;
                    background: #53D769;
                }

                Text {
                    text: root.status;
                    color: #8A90A0;
                    font-size: 12px;
                    vertical-alignment: center;
                    horizontal-stretch: 1;
                    overflow: elide;
                }
            }
        }
    }
}

use slint::{ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

fn main() -> Result<(), slint::PlatformError> {
    let config = config::Config::load();
    let loaded = packs::load_packs();

    let window = AppWindow::new()?;

    let active = loaded
        .iter()
        .position(|pack| pack.manifest.id == config.active_pack)
        .unwrap_or(0);

    window.set_sound_enabled(config.sound_enabled);
    window.set_muted(config.muted);
    set_pack_models(&window, &loaded);
    window.set_current_pack(active as i32);
    show_pack(&window, loaded.get(active));

    let audio = audio::AudioHandle::spawn();
    let engine = Arc::new(hooks::Engine::new(audio.clone()));
    engine.set_flags(config.sound_enabled, config.muted);
    if let Some(pack) = loaded.get(active) {
        engine.set_pack(pack);
    }
    hooks::start(engine.clone());

    window.set_status(initial_status(loaded.len()).into());

    let packs = Rc::new(RefCell::new(loaded));

    let weak = window.as_weak();

    window.on_select_pack({
        let weak = weak.clone();
        let packs = packs.clone();
        let engine = engine.clone();
        move |index| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let idx = index.max(0) as usize;
            let packs = packs.borrow();
            let Some(pack) = packs.get(idx) else {
                return;
            };
            window.set_current_pack(index);
            show_pack(&window, Some(pack));
            engine.set_pack(pack);
            save_config(&window, &packs, &engine);
        }
    });

    window.on_apply({
        let weak = weak.clone();
        let packs = packs.clone();
        let engine = engine.clone();
        move || {
            let Some(window) = weak.upgrade() else {
                return;
            };
            save_config(&window, &packs.borrow(), &engine);
        }
    });

    window.on_preview({
        let weak = weak.clone();
        let packs = packs.clone();
        let engine = engine.clone();
        move || {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let idx = window.get_current_pack().max(0) as usize;
            let name = packs
                .borrow()
                .get(idx)
                .map(|pack| pack.manifest.name.clone())
                .unwrap_or_else(|| "selected pack".to_string());

            match engine.preview() {
                Ok(()) => window.set_status(format!("Playing {name}").into()),
                Err(err) => window.set_status(err.into()),
            }
        }
    });

    window.on_reload_packs({
        let weak = weak.clone();
        let packs = packs.clone();
        let engine = engine.clone();
        move || {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut reloaded = packs::load_packs();
            let count = reloaded.len();
            set_pack_models(&window, &reloaded);

            let idx = (window.get_current_pack().max(0) as usize).min(count.saturating_sub(1));
            window.set_current_pack(idx as i32);
            show_pack(&window, reloaded.get(idx));
            if let Some(pack) = reloaded.get(idx) {
                engine.set_pack(pack);
            }

            *packs.borrow_mut() = std::mem::take(&mut reloaded);
            window.set_status(format!("Reloaded {count} pack(s) - live input active").into());
        }
    });

    window.on_open_data({
        let weak = weak.clone();
        move || {
            let result = std::process::Command::new("explorer")
                .arg(packs::packs_dir())
                .spawn();
            if let Err(err) = result {
                if let Some(window) = weak.upgrade() {
                    window.set_status(format!("Could not open packs folder: {err}").into());
                }
            }
        }
    });

    window.on_show_about({
        let weak = weak.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                window.set_status(
                    "Ticky 0.1.0 - built with Rust and Slint. Sounds are CC0.".into(),
                );
            }
        }
    });

    window.on_quit_app({
        let weak = weak.clone();
        let audio = audio.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let _ = window.hide();
            }
            audio.shutdown();
            let _ = slint::quit_event_loop();
        }
    });

    window.run()
}

fn initial_status(pack_count: usize) -> String {
    if pack_count == 0 {
        "No sound packs found - add a folder with pack.toml to assets/packs".to_string()
    } else if thockly_running() {
        "Thockly is running - quit it to avoid double sounds".to_string()
    } else {
        format!("Live input active - {pack_count} sound pack(s) loaded")
    }
}

fn thockly_running() -> bool {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq Thockly.exe", "/NH"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .to_ascii_lowercase()
                .contains("thockly.exe")
        })
        .unwrap_or(false)
}

fn set_pack_models(window: &AppWindow, packs: &[packs::Pack]) {
    let names: Vec<SharedString> = packs
        .iter()
        .map(|pack| pack.manifest.name.clone().into())
        .collect();
    let authors: Vec<SharedString> = packs
        .iter()
        .map(|pack| format!("by {}", pack.manifest.author).into())
        .collect();
    window.set_pack_names(ModelRc::new(VecModel::from(names)));
    window.set_pack_authors(ModelRc::new(VecModel::from(authors)));
}

fn show_pack(window: &AppWindow, pack: Option<&packs::Pack>) {
    match pack {
        Some(pack) => {
            window.set_current_pack_author(pack.manifest.author.clone().into());
            window.set_current_pack_license(pack.manifest.license.clone().into());
            window.set_current_pack_description(pack.manifest.description.clone().into());
        }
        None => {
            window.set_current_pack_author("".into());
            window.set_current_pack_license("".into());
            window.set_current_pack_description(
                "No sound packs found. Add a folder with a pack.toml to assets/packs.".into(),
            );
        }
    }
}

fn save_config(window: &AppWindow, packs: &[packs::Pack], engine: &hooks::Engine) {
    let idx = window.get_current_pack().max(0) as usize;
    let sound_enabled = window.get_sound_enabled();
    let muted = window.get_muted();

    engine.set_flags(sound_enabled, muted);

    let config = config::Config {
        active_pack: packs
            .get(idx)
            .map(|pack| pack.manifest.id.clone())
            .unwrap_or_default(),
        sound_enabled,
        muted,
    };

    match config.save() {
        Ok(()) => {
            let name = packs
                .get(idx)
                .map(|pack| pack.manifest.name.as_str())
                .unwrap_or("settings");
            window.set_status(format!("Saved {name} - live input active").into());
        }
        Err(err) => window.set_status(format!("Save failed: {err}").into()),
    }
}
