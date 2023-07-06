use gtk::prelude::*;
use gtk::{
    Application, 
    ApplicationWindow,
    glib
};

use std::{
    sync::{Arc, Mutex},
};

use crate::stage_control::control::{wait_for_motor_idle, set_high_speed};
use crate::stage_control::{
    send_command_get_response,
};

const APP_ID: &str = "org.gtk_rs.InteractWindow";


pub fn make_interact_window(handle: &Arc<Mutex<rusb::DeviceHandle<rusb::GlobalContext>>>) -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate({
        let handle_copy = Arc::clone(handle);
        move |app : &Application| {
            build_interact_ui(app, &handle_copy);
        }
    });

    app.run()
}

fn make_entry(placeholder: &str, margin: i32) -> gtk::Entry {
    return gtk::Entry::builder()
        .placeholder_text(placeholder)
        .margin_bottom(margin)
        .margin_end(margin)
        .margin_start(margin)
        .margin_top(margin)
        .build();
}

fn make_button(label: &str, margin: i32) -> gtk::Button {
    return gtk::Button::builder()
        .label(label)
        .margin_bottom(margin)
        .margin_end(margin)
        .margin_start(margin)
        .margin_top(margin)
        .build();
}

fn make_label(label: &str) -> gtk::Label {
    return gtk::Label::builder()
        .label(label)
        .build();
}


fn build_interact_ui(application: &Application, handle: &Arc<Mutex<rusb::DeviceHandle<rusb::GlobalContext>>>) {
    let general_entry = make_entry("Input Command", 12);

    let hspd_entry = make_entry("Input high speed", 12);

    let output_label = make_label("Output:");
    output_label.set_halign(gtk::Align::Start);

    let output = make_label("");
    output.set_margin_bottom(1);
    output.set_margin_top(1);
    output.set_margin_start(1);
    output.set_margin_end(1);


    let j_plus_button = make_button("Move up", 12);

    let stop_button = make_button("Stop", 12);

    let j_minus_button = make_button("Move down", 12);


    let grid = gtk::Grid::builder().build();
    grid.attach(&general_entry, 0, 0, 1, 1);
    grid.attach(&output_label, 0, 1, 1, 1);
    grid.attach(&output, 1, 1, 1, 1);
    grid.attach(&j_plus_button, 3, 0, 1, 1);
    grid.attach(&stop_button, 3, 1, 1, 1);
    grid.attach(&j_minus_button, 3, 2, 1, 1);
    grid.attach(&hspd_entry, 3, 3, 1, 1);

    general_entry.connect_activate({
        let handle_clone = Arc::clone(&handle);
        move |general_entry| {
            let mut mutex_handle = handle_clone.lock().unwrap();

            let command = general_entry.text().to_string().to_ascii_uppercase();

            match send_command_get_response(&mut mutex_handle, &[command.as_bytes(), b"\0"].concat()) {
                Ok(r) => output.set_text(&r),
                Err(_e) => output.set_text(&std::format!("Command '{}' not understood", command)),
            };
            wait_for_motor_idle(&mut mutex_handle, None, None).unwrap();
            
            general_entry.set_text("");
        }
    });

    hspd_entry.connect_activate({
        let handle_clone = Arc::clone(&handle);
        move |hspd_entry| {
            let mut mutex_handle = handle_clone.lock().unwrap();

            let hspd: u64 = match hspd_entry.text().to_string().parse() {
                Ok(num) => num,
                Err(_e) => {
                    hspd_entry.set_text("Input a number!"); 
                    return;
                },
            };

            match set_high_speed(&mut mutex_handle, hspd) {
                Ok(()) => hspd_entry.set_text(""),
                Err(e) => hspd_entry.set_text(&std::format!("Failed with error {:?}", e)),
            };
        }
    });

    j_plus_button.connect_clicked({
        let handle_clone = Arc::clone(&handle);
        move |_| {
            let mut mutex_handle = handle_clone.lock().unwrap();
        
            match send_command_get_response(&mut mutex_handle, b"J+\0") {
                Ok(_r) => (),
                Err(e) => println!("Something when wrong! {}", e),

            };
        }
    });

    j_minus_button.connect_clicked({
        let handle_clone = Arc::clone(&handle);
        move |_| {
            let mut mutex_handle = handle_clone.lock().unwrap();
        
            match send_command_get_response(&mut mutex_handle, b"J-\0") {
                Ok(_r) => (),
                Err(e) => println!("Something when wrong! {}", e),
            };
        }
    });

    stop_button.connect_clicked({
        let handle_clone = Arc::clone(&handle);
        move |_| {
            let mut mutex_handle = handle_clone.lock().unwrap();
        
            match send_command_get_response(&mut mutex_handle, b"STOP\0") {
                Ok(_r) => (),
                Err(e) => println!("Something when wrong! {}", e),
            };
        }
    });

    // Create a window
    let window = ApplicationWindow::builder()
        .application(application)
        .default_height(200)
        .default_width(500)
        .title("My GTK App")
        .child(&grid)
        .build();

    window.connect_destroy({
        let handle_clone = Arc::clone(&handle); 
        move |_| {
            let mut mutex_handle = handle_clone.lock().unwrap();

            match send_command_get_response(&mut mutex_handle, b"STOP\0") {
                Ok(_r) => (),
                Err(e) => println!("Something when wrong! {}", e),
            };
        }
    });

    // Present the window
    window.present();
}