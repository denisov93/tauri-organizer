#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::fs::File;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{Write, Read};

use tauri::{
    api::shell::open, CustomMenuItem,
    SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, SystemTraySubmenu
};
use tauri::{Manager, GlobalWindowEvent};
use cli_clipboard;

use serde::{Deserialize, Serialize};
use serde_json::Result;

const FILE_PATH: &str = "../dist/link_list.json";

const LINKS: [(&str, &str, &str); 7] = [
    // social LINKS
    ("open-social-prod", "Bloqit Prod","https://admin.bloq.it/en/dashboard"),
    ("open-social-dev", "Bloqit Dev","https://admin.dev.bloq.it/en/dashboard"),
    ("open-social-twitter", "LinkedIn","https://www.linkedin.com/in/alexander-denisov-2a89771b8/"),
    // github LINKS
    ("open-github-my", "My GitHub","https://github.com/denisov93"),
    ("open-github-vue", "Bloqit Vue","https://github.com/bloqit/vue-backoffice"),
    ("open-github-node", "Bloqit Node","https://github.com/bloqit/nodejs-backend"),
    ("open-github-rust-adventure", "Rust Adventure Example","https://github.com/rust-adventure/yt-tauri-menubar-example"),
];

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_links() -> Vec<Link> {
    dbg!("get_links");
    let mut list = ListLinks::new();
    let mut file = OpenOptions::new().write(true).read(true).open(FILE_PATH).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("error");
    list = serde_json::from_str(&contents).unwrap();
    dbg!(&list.links);
    list.links
}

#[tauri::command]
fn update_list_of_links(links: Vec<Link>) -> String {
    dbg!("update_list_of_links");
    let mut file = OpenOptions::new().write(true).read(true).open(FILE_PATH).unwrap();
    let mut list = ListLinks::new();
    list.links = links;
    let j = serde_json::to_string(&list).unwrap();
    file.write_all(j.as_bytes()).expect("error");
    drop(file);
    "ok".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
struct ListLinks {
    links: Vec<Link>
}

impl ListLinks {
    pub fn new() -> Self {
        Self {
            links: Vec::new()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Link {
    title: String,
    url: String
}

impl Link {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            url: String::new()
        }
    }
    
}

pub struct History {
    clipboard_history: Mutex<Vec<String>>,
    flag: Mutex<bool>
}


impl History {
    pub fn new() -> Self {
        Self {
            clipboard_history: Mutex::new(Vec::new()),
            flag: Mutex::new(false)
        }
    }

    pub fn get(&self) -> Vec<String> {
        self.clipboard_history.lock().unwrap().clone()
    }
    
}

fn main() {


    thread::spawn(move|| {
        let mut file;
        let mut list = ListLinks::new();
        
        match Path::new(FILE_PATH).try_exists() {
            Ok(true) => {
                file = OpenOptions::new().write(true).read(true).open(FILE_PATH).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).expect("error");
                list = serde_json::from_str(&contents).unwrap();
                println!("{:?}", list.links.first())
            },
            _ => {
                let mut link = Link::new();
                link.title = "Test".to_string();
                link.url = "https://www.google.com".to_string();
                list.links.push(link);
                let j = serde_json::to_string(&list).unwrap();

                file = OpenOptions::new().write(true).read(true).create(true).open(FILE_PATH).unwrap();   
                file.write_all(j.as_bytes()).expect("error");
            }

        }
        drop(file);
        thread::sleep(Duration::from_millis(1000));
    });

    
    // let mut contents = String::new();
    // file.read_to_string(&mut contents).expect("error");
    
    // let mut list: ListLinks = serde_json::from_str(&contents).unwrap();

    
    

    let history = Arc::new(History::new());
    let cl1 = history.clone();
    let cl3 = history.clone();
        
    let tray_menu = build_system_tray_menu(cl1.get());
    let tray = SystemTray::new().with_menu(tray_menu);

    let guard = Arc::new(Mutex::new(mpsc::channel()));

    let tx_clone = guard.clone();
    
    thread::spawn(move|| {
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
                },
                _=> {}
            }
            thread::sleep(Duration::from_millis(1000));
        }
    });


    tauri::Builder::default()
        .system_tray(tray)
        .setup(move |app|{
            let app_handle = app.app_handle();
            tauri::async_runtime::spawn(async move {
                loop {
                    let clipboard = guard.lock().unwrap().1.try_recv();
                    match clipboard {
                        Ok(_clipboard) => {
                            // println!("Received a message from the thread");
                            let tray_menu = build_system_tray_menu(cl3.get());
                            app_handle.tray_handle().set_menu(tray_menu).unwrap();
                        },
                        Err(_) => {}
                    }
                    thread::sleep(Duration::from_millis(1000));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet,get_links,update_list_of_links])
        // .on_system_tray_event(on_system_tray_event)
        .on_system_tray_event(move | app,event| { 
            match event {
                SystemTrayEvent::MenuItemClick { id, .. } => {
                    let item_handle =
                        app.tray_handle().get_item(&id);
                    // dbg!(&id);
                    match id.as_str() {
                        "visibility-toggle" => {
                            let window =
                                app.get_window("main").unwrap();
                            match window.is_visible() {
                                Ok(true) => {
                                window.hide().unwrap();
                                item_handle.set_title("Show").unwrap();
                                },
                                Ok(false) => {
                                window.show().unwrap();
                                item_handle.set_title("Hide").unwrap();
        
                                },
                                Err(_e) => unimplemented!("what kind of errors happen here?"),
                            }
                        }
                        "clear-history" => {
                            cl1.clipboard_history.lock().unwrap().clear();                            
                            let tray_menu = build_system_tray_menu(vec![]);
                            app.tray_handle().set_menu(tray_menu).unwrap();
                        }
                        "quit" => app.exit(0),
                        s if s.starts_with("open-") => {
                            if let Some(link) = LINKS
                                .iter()
                                .find(|(id, ..)| id == &s)
                            {
                                open(
                                    &app.shell_scope(),
                                    link.2,
                                    None,
                                )
                                .unwrap();
                            }
                        }, 
                        s => {
                            cli_clipboard::set_contents(s.to_string()).unwrap();
                            cl1.clipboard_history.lock().unwrap().retain(|x| x != s);
                            let tray_menu = build_system_tray_menu(cl1.get());
                            app.tray_handle().set_menu(tray_menu).unwrap();
                        }
                        // _ => {}
                    }
                }
                _ => {}
            }

            
        })
        .on_window_event(on_window_event)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


fn on_window_event(
    event: GlobalWindowEvent
) {
    match event.event() {
        tauri::WindowEvent::CloseRequested { api, .. } => {
            // don't kill the app when the user clicks close. this is important
            event.window().hide().unwrap();
            api.prevent_close();
            event.window().app_handle().tray_handle().get_item("visibility-toggle").set_title("Show").unwrap();
        },
        _ => {}
    }
}

fn build_system_tray_menu( clipboard_history: Vec<String>) -> SystemTrayMenu {
    let copy_paste_menu = {
        let mut menu = SystemTrayMenu::new();

        if clipboard_history.len() > 0 {
            for item in clipboard_history.iter() {
                let mut title = item.to_string();
                if item.len() > 20 {
                    title = item[0..20].to_string();
                    title.push_str(" ...");
                }
                menu = menu.add_item(CustomMenuItem::new(
                    item.to_string(),
                    title.to_string(),
                ));
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
        for (id, _label, _url) in
            LINKS.iter().filter(|(id, _label, _url)| {
                id.starts_with("open-social")
            })
        {
            menu = menu.add_item(CustomMenuItem::new(
                id.to_string(),
                _label.to_string(),
            ));
        }
        
        SystemTraySubmenu::new("Social", menu)
    };

    let sub_menu_github = {
        let mut menu = SystemTrayMenu::new();
        for (id, _label, _url) in
            LINKS.iter().filter(|(id, _label, _url)| {
                id.starts_with("open-github")
            })
        {
            menu = menu.add_item(CustomMenuItem::new(
                id.to_string(),
                _label.to_string(),
            ));
        }

        SystemTraySubmenu::new("GitHub", menu)
    };
 
    SystemTrayMenu::new()
        .add_submenu(copy_paste_menu)
        .add_submenu(sub_menu_social)
        .add_submenu(sub_menu_github)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new(
            "visibility-toggle".to_string(),
            "Hide",
        ))
        .add_item(CustomMenuItem::new(
            "quit".to_string(),
            "Quit",
        ))
}