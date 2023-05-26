#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::fs::{self,OpenOptions};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cli_clipboard;
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    api::shell::open, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, SystemTraySubmenu,
    GlobalWindowEvent, Manager,
};

extern crate directories;

static GLOBAL_FLAG: AtomicBool = AtomicBool::new(false);

const FILE_PATH: &str = "link_list.json"; // file name

// Hardcoded as placeholders for now
const LINKS: [(&str, &str, &str); 3] = [
    // social LINKS
    (
        "open-social-linkedIn",
        "LinkedIn",
        "https://www.linkedin.com/in/alexander-denisov-2a89771b8/",
    ),
    // github LINKS
    (
        "open-github-my",
        "My GitHub",
        "https://github.com/denisov93",
    ),
    (
        "open-github-tauri",
        "Tauri GitHub",
        "https://github.com/tauri-apps/tauri",
    )
];

pub fn set_flag_to_true() {
    GLOBAL_FLAG.store(true, Ordering::SeqCst);
}

pub fn set_flag_to_false() {
    GLOBAL_FLAG.store(false, Ordering::SeqCst);
}

pub fn get_flag() -> bool {
    GLOBAL_FLAG.load(Ordering::SeqCst)
}

pub fn get_flag_and_set_to_true() -> bool {
    GLOBAL_FLAG.swap(true, Ordering::SeqCst)
}

pub fn get_path() -> std::path::PathBuf {
    let app_dirs = AppDirs::new(Some("tauri-organizer"), true).unwrap();
    let config_file_path = app_dirs.config_dir.join(FILE_PATH);
    config_file_path
}

#[tauri::command]
fn get_links_location() -> String {
    let path = get_path();
    path.to_str().unwrap().to_string()
}

#[tauri::command]
fn get_links() -> Vec<Link> {
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(get_path())
        .unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("error");
    let list: ListLinks = serde_json::from_str(&contents).unwrap();
    list.links
}

#[tauri::command]
fn update_list_of_links(links: Vec<Link>) -> String {
    let path = get_path();
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .read(true)
        .open(path)
        .unwrap();
    let mut list = ListLinks::new();
    list.links = links;
    let j = serde_json::to_string(&list).unwrap();
    file.write_all(j.as_bytes()).expect("error");

    return "ok".to_string();
}

#[derive(Serialize, Deserialize, Debug)]
struct ListLinks {
    links: Vec<Link>,
}

impl ListLinks {
    pub fn new() -> Self {
        Self { links: Vec::new() }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Link {
    id: String,
    title: String,
    url: String,
}

impl Link {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            url: String::new(),
        }
    }
}

pub struct History {
    clipboard_history: Mutex<Vec<String>>,
    flag: Mutex<bool>,
}

impl History {
    pub fn new() -> Self {
        Self {
            clipboard_history: Mutex::new(Vec::new()),
            flag: Mutex::new(false),
        }
    }

    pub fn get(&self) -> Vec<String> {
        self.clipboard_history.lock().unwrap().clone()
    }
}

fn get_list_from_file() -> Vec<Link> {
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(get_path())
        .unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("error");
    let list: ListLinks = serde_json::from_str(&contents).unwrap();
    list.links
}

fn main() {
    let mut list = ListLinks::new();

    let app_dirs = AppDirs::new(Some("tauri-organizer"), true).unwrap();
    let config_file_path = app_dirs.config_dir.join(FILE_PATH);

    fs::create_dir_all(&app_dirs.config_dir).unwrap();

    let _file = if config_file_path.exists() {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(config_file_path);
        match file {
            Ok(mut f) => {
                let mut contents = String::new();
                f.read_to_string(&mut contents).expect("error");
                list = serde_json::from_str(&contents).unwrap();
            }
            Err(e) => panic!("Error opening file: {}", e),
        }
    } else {
        let mut link = Link::new();
        link.id = "links-google".to_string();
        link.title = "google".to_string();
        link.url = "https://www.google.com".to_string();
        list.links.push(link);
        let j = serde_json::to_string(&list).unwrap();

        match OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(config_file_path)
        {
            Ok(mut f) => {
                f.write_all(j.as_bytes()).expect("error");
            }
            Err(e) => panic!("Error opening file: {}", e),
        }
    };

    let history = Arc::new(History::new());
    let cl1 = history.clone();
    let cl3 = history.clone();

    let tray_menu = build_system_tray_menu(cl1.get());
    let tray = SystemTray::new().with_menu(tray_menu);

    let guard = Arc::new(Mutex::new(mpsc::channel()));

    let tx_clone = guard.clone();

    thread::spawn(move || {
        let mut last_copy = String::new();
        loop {
            let cl2 = history.clone();
            let mut is_cloned = false;
            let copy = cli_clipboard::get_contents();
            match copy {
                Ok(copy) => {
                    if last_copy != copy {
                        last_copy = copy.clone();
                        cl2.clipboard_history.lock().unwrap().push(copy.clone());
                        let mut flag = cl2.flag.lock().unwrap();
                        *flag = true;
                        is_cloned = true;
                    }

                    if is_cloned {
                        tx_clone.lock().unwrap().0.send(cl2.get()).unwrap();
                    }
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(1000));
        }
    });

    tauri::Builder::default()
        .system_tray(tray)
        .setup(move |app| {
            let app_handle = app.app_handle();
            tauri::async_runtime::spawn(async move {
                loop {
                    let clipboard = guard.lock().unwrap().1.try_recv();
                    match clipboard {
                        Ok(_clipboard) => {
                            // println!("Received a message from the thread");
                            let tray_menu = build_system_tray_menu(cl3.get());
                            app_handle.tray_handle().set_menu(tray_menu).unwrap();
                        }
                        Err(_) => {}
                    }
                    thread::sleep(Duration::from_millis(1000));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_links_location,
            get_links,
            update_list_of_links
        ])
        // .on_system_tray_event(on_system_tray_event)
        .on_system_tray_event(move |app, event| {
            if get_flag() {
                let tray_menu = build_system_tray_menu(cl1.get());
                app.tray_handle().set_menu(tray_menu).unwrap();
                set_flag_to_false();
            }
            match event {
                SystemTrayEvent::MenuItemClick { id, .. } => {
                    let item_handle = app.tray_handle().get_item(&id);
                    // dbg!(&id);
                    match id.as_str() {
                        "visibility-toggle" => {
                            let window = app.get_window("main").unwrap();
                            match window.is_visible() {
                                Ok(true) => {
                                    window.hide().unwrap();
                                    item_handle.set_title("Show").unwrap();
                                }
                                Ok(false) => {
                                    window.show().unwrap();
                                    item_handle.set_title("Hide").unwrap();
                                }
                                Err(_e) => unimplemented!("what kind of errors happen here?"),
                            }
                        }
                        "clear-history" => {
                            cl1.clipboard_history.lock().unwrap().clear();
                            let tray_menu = build_system_tray_menu(vec![]);
                            app.tray_handle().set_menu(tray_menu).unwrap();
                        }
                        "quit" => app.exit(0),
                        s if s.starts_with("links-") => {
                            let mut list = get_list_from_file();
                            for link in list.iter_mut() {
                                if link.id == s {
                                    open(&app.shell_scope(), link.url.clone(), None).unwrap();
                                }
                            }
                        }
                        s if s.starts_with("open-") => {
                            if let Some(link) = LINKS.iter().find(|(id, ..)| id == &s) {
                                open(&app.shell_scope(), link.2, None).unwrap();
                            }
                        }
                        s => {
                            cli_clipboard::set_contents(s.to_string()).unwrap();
                            cl1.clipboard_history.lock().unwrap().retain(|x| x != s);
                            let tray_menu = build_system_tray_menu(cl1.get());
                            app.tray_handle().set_menu(tray_menu).unwrap();
                        } // _ => {}
                    }
                }
                _ => {}
            }
        })
        .on_window_event(on_window_event)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn on_window_event(event: GlobalWindowEvent) {
    match event.event() {
        tauri::WindowEvent::Focused(false) => {
            set_flag_to_true();
        }
        tauri::WindowEvent::CloseRequested { api, .. } => {
            // don't kill the app when the user clicks close. this is important
            event.window().hide().unwrap();
            api.prevent_close();
            event
                .window()
                .app_handle()
                .tray_handle()
                .get_item("visibility-toggle")
                .set_title("Show")
                .unwrap();
        }
        _ => {}
    }
}

fn build_system_tray_menu(clipboard_history: Vec<String>) -> SystemTrayMenu {
    let copy_paste_menu = {
        let mut menu = SystemTrayMenu::new();

        if clipboard_history.len() > 0 {
            for item in clipboard_history.iter() {
                let mut title = item.to_string();
                if item.len() > 20 {
                    title = item[0..20].to_string();
                    title.push_str(" ...");
                }
                menu = menu.add_item(CustomMenuItem::new(item.to_string(), title.to_string()));
            }
        }
        menu = menu.add_native_item(SystemTrayMenuItem::Separator);
        menu = menu.add_item(CustomMenuItem::new(
            "clear-history".to_string(),
            "Clear History".to_string(),
        ));

        SystemTraySubmenu::new("Copy-Paste", menu)
    };

    let sub_menu_social = {
        let mut menu = SystemTrayMenu::new();
        for (id, _label, _url) in LINKS
            .iter()
            .filter(|(id, _label, _url)| id.starts_with("open-social"))
        {
            menu = menu.add_item(CustomMenuItem::new(id.to_string(), _label.to_string()));
        }

        SystemTraySubmenu::new("Social", menu)
    };

    let sub_menu_github = {
        let mut menu = SystemTrayMenu::new();
        for (id, _label, _url) in LINKS
            .iter()
            .filter(|(id, _label, _url)| id.starts_with("open-github"))
        {
            menu = menu.add_item(CustomMenuItem::new(id.to_string(), _label.to_string()));
        }

        SystemTraySubmenu::new("GitHub", menu)
    };

    let sub_menu_links = {
        let mut menu = SystemTrayMenu::new();
        for link in get_list_from_file().iter() {
            menu = menu.add_item(CustomMenuItem::new(
                link.id.to_string(),
                link.title.to_string(),
            ));
        }

        SystemTraySubmenu::new("Links", menu)
    };

    SystemTrayMenu::new()
        .add_submenu(copy_paste_menu)
        .add_submenu(sub_menu_links)
        .add_submenu(sub_menu_social)
        .add_submenu(sub_menu_github)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("visibility-toggle".to_string(), "Hide"))
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"))
}
