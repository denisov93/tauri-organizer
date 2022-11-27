#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_links() -> String {
    format!("Version: {:#?}", LINKS)
}

use tauri::{
    api::shell::open, AppHandle, CustomMenuItem,
    SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, SystemTraySubmenu,
};

use tauri::Manager;

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

fn main() {
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
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new(
            "quit".to_string(),
            "Quit",
        ))
        .add_submenu(sub_menu_social)
        .add_submenu(sub_menu_github)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new(
            "visibility-toggle".to_string(),
            "Hide",
        ));

    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(tray)
        .invoke_handler(tauri::generate_handler![greet,get_links])
        .on_system_tray_event(on_system_tray_event)
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                // don't kill the app when the user clicks close. this is important
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn on_system_tray_event(
    app: &AppHandle,
    event: SystemTrayEvent,
) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
            let item_handle =
                app.tray_handle().get_item(&id);
            dbg!(&id);
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
                }
                _ => {}
            }
        }
        _ => {}
    }
}