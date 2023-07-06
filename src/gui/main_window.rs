use gtk::{
    prelude::*,
    Application,
    ApplicationWindow,
    glib,
};

use std::{
    time::Instant,
    sync::{Arc, Mutex},
    io::Write,
};

use crate::{
    stage_control::{
        open,
        close,
        calibrate,
        run,
    },
    gui::interact_window,
};

const APP_ID: &str = "org.gtk_rs.Main";

struct MainWindow {
    arc_mutex_handle: Arc<Mutex<rusb::DeviceHandle<rusb::GlobalContext>>>,
    calibrate_button: gtk::Button,
    run_button: gtk::Button,
    interact_button: gtk::Button,
    output_entry: Arc<gtk::Entry>,
    _grid: gtk::Grid,
    window: ApplicationWindow,
}

impl MainWindow {
    fn new(application: &Application, arc_mutex_handle: Arc<Mutex<rusb::DeviceHandle<rusb::GlobalContext>>>) -> Self {
        let calibrate_button = Self::make_button("Calibrate", 12);
        let run_button = Self::make_button("Run", 12);
        let interact_button = Self::make_button("Interact", 12);
        let output_entry = Arc::new(Self::make_entry("Output", 12));

        let _grid = gtk::Grid::builder().hexpand_set(true).vexpand_set(true).build();
        _grid.attach(&calibrate_button, 0, 0, 1, 1);
        _grid.attach(&run_button, 0, 1, 1, 1);
        _grid.attach(&interact_button, 0, 2, 1, 1);
        _grid.attach(output_entry.as_ref(), 1, 0, 2, 3);

        let window = ApplicationWindow::builder()
            .application(application)
            .title("Main window")
            .child(&_grid)
            .default_height(300)
            .default_width(800)
            .build();

        Self {
            arc_mutex_handle,
            calibrate_button,
            run_button,
            interact_button,
            output_entry,
            _grid,
            window,
        }
    }

    fn make_entry(placeholder: &str, margin: i32) -> gtk::Entry {
        gtk::Entry::builder()
            .placeholder_text(placeholder)
            .margin_bottom(margin)
            .margin_end(margin)
            .margin_start(margin)
            .margin_top(margin)
            .vexpand(true)
            .hexpand(true)
            .build()
    }

    fn make_button(label: &str, margin: i32) -> gtk::Button {
        gtk::Button::builder()
            .label(label)
            .margin_bottom(margin)
            .margin_end(margin)
            .margin_start(margin)
            .margin_top(margin)
            .vexpand(true)
            .hexpand(true)
            .build()
    }

    fn connect_calibrate_button(&self) {
        let arc_mutex_handle_clone = Arc::clone(&self.arc_mutex_handle);
        let entry = Arc::clone(&self.output_entry);

        self.calibrate_button.connect_clicked(move |_| {
            let mut mutex_handle = arc_mutex_handle_clone.lock().unwrap();

            match calibrate::calibrate(&mut mutex_handle) {
                Ok(()) => entry.set_text("Calibration successful."),
                Err(e) => entry.set_text(&format!("Something went wrong! Error was {:?}", e)),
            };
        });
    }

    fn connect_run_button(&self) {
        let arc_mutex_handle_clone = Arc::clone(&self.arc_mutex_handle);
        let output_entry = Arc::clone(&self.output_entry);

        self.run_button.connect_clicked(move |_| {
            output_entry.set_text("Use interactive mode to set the stage.");
            interact_window::make_interact_window(&arc_mutex_handle_clone);

            let mut mutex_handle = arc_mutex_handle_clone.lock().unwrap();

            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open("./RunOutput.dat")
                .unwrap();

            let params: run::RunParameters = run::run_prep(&mut mutex_handle).unwrap();
            for _ in 1..params.load_standby_cyles + 1 {
                output_entry.set_text("HDHELRFJKDLKHJDF:LKJDF");

                match run::run(&mut mutex_handle, &params, &mut file) {
                    Ok(()) => (),
                    Err(e) => output_entry.set_text(&format!("Something went wrong! Error was {:?}", e)),
                };

                let time = Instant::now();
                output_entry.set_text(
                    "Loading cycle complete. Use interactive mode if needed.
                    Close and motor will return to zero until next cycle.
                    DO NOT change the pulse position (PX=##), you will break the system."
                );

                // drop(mutex_handle);
                // interact_window::make_interact_window(&arc_mutex_handle_clone);
                // mutex_handle = arc_mutex_handle_clone.lock().unwrap();

                while time.elapsed().as_secs() < params.standby_duration {}
            }
        });
    }

    fn connect_interact_button(&self) {
        let arc_mutex_handle_clone = Arc::clone(&self.arc_mutex_handle);

        self.interact_button.connect_clicked(move |_| {
            interact_window::make_interact_window(&arc_mutex_handle_clone);
        });
    }

    fn connect_window_destroy(&self) {
        let arc_mutex_handle_clone = Arc::clone(&self.arc_mutex_handle);

        self.window.connect_destroy(move |_| {
            close(&mut arc_mutex_handle_clone.lock().unwrap()).unwrap();
        });
    }

    fn show(&self) {
        self.window.present();
    }
}

fn build_main_ui(application: &Application) {
    let arc_mutex_handle = match open(0x1589, 0xa101) {
        Ok(handle) => Arc::new(Mutex::new(handle)),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function stage_control::driver::open failed due to error {} in gui::main_window::build_main_ui", e).unwrap();
            return build_error_window(application);
        },
    };

    let main_window = MainWindow::new(application, Arc::clone(&arc_mutex_handle));
    main_window.connect_calibrate_button();
    main_window.connect_run_button();
    main_window.connect_interact_button();
    main_window.connect_window_destroy();
    main_window.show();
}

fn build_error_window(application: &Application) {
    let label = gtk::Label::builder()
        .vexpand(true)
        .hexpand(true)
        .label("HELLO")
        .build();

    let grid = gtk::Grid::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    grid.attach(&label, 0, 0, 1, 1);

    let window = ApplicationWindow::builder()
        .application(application)
        .title("Error window")
        .child(&grid)
        .build();

    window.present();
}

pub fn make_main_window() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        build_main_ui(app);
    });

    app.run()
}
