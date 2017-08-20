use std::borrow::Cow;
use std::path::Path;
use std::cell::RefCell;
use std::rc::Rc;

use clap::App;
use gtk;
use gdk::prelude::ContextExt;
use gtk::{Builder, Window, ListStore, FileChooserButton, DrawingArea, TreeSelection};
use gtk::prelude::*;
use gdk_pixbuf::{Pixbuf, InterpType};
use cairo;

use common::config;

pub const SUBCOMMAND_NAME: &str = "stats-analyzer";

pub fn subcommand() -> App<'static, 'static> {
    App::new(SUBCOMMAND_NAME)
        .about("Starts the GUI-based analyzer of the collected statistics about wallpapers")
}

macro_rules! cloned {
    ($($name:ident),+; $f:expr) => {{
        $(let $name = $name.clone();)+
        $f
    }}
}

pub fn main(config_path: Cow<Path>) {
    if gtk::init().is_err() {
        abort!(1, "Failed to initialize GTK");
    }

    let config = config::load(&config_path)
        .unwrap_or_else(|e| abort!(1, "Cannot load configuration file {}: {}", config_path.display(), e));

    let main_window_src = include_str!("main_window.glade");
    let builder = Builder::new_from_string(main_window_src);

    let window: Window = builder.get_object("main_window").unwrap();

    let state = Rc::new(State {
        images_store: builder.get_object("images_store").unwrap(),
        database_file_chooser: builder.get_object("database_file").unwrap(),
        preview_image: builder.get_object("preview_image").unwrap(),
        buffer_pixbuf: RefCell::new(None),
        image_selection: builder.get_object("image_selection").unwrap(),
    });

    if let Some(configured_path) = config.server.stats_db {
        state.database_file_chooser.set_filename(&configured_path);
        state.reload_database();
    }

    state.database_file_chooser.connect_file_set(cloned!(state; move |_| {
        state.reload_database();
    }));

    state.image_selection.connect_changed(cloned!(state; move |_| {
        state.load_image();
    }));

    state.preview_image.connect_draw(cloned!(state; move |_, ctx| {
        state.rescale_image(ctx);
        Inhibit(false)
    }));

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window.show_all();

    gtk::main();
}

struct State {
    images_store: ListStore,
    database_file_chooser: FileChooserButton,
    buffer_pixbuf: RefCell<Option<Pixbuf>>,
    preview_image: DrawingArea,
    image_selection: TreeSelection,
}

impl State {
    fn reload_database(&self) {
        if let Some(db_path) = self.database_file_chooser.get_filename() {
            use diesel::prelude::*;
            use diesel::sqlite::SqliteConnection;
            use daemon::stats::model::ImageStatistics;
            use daemon::stats::schema::image_statistics::dsl::*;

            let conn = SqliteConnection::establish(&db_path.as_os_str().to_string_lossy())
                .expect("Couldn't open image statistics database");

            let data = image_statistics
                .order(filename.asc())
                .load::<ImageStatistics>(&conn)
                .expect("Couldn't load image statistics");

            self.images_store.clear();

            for image in data {
                let iter = self.images_store.append();
                let total_display_time_str = format_display_time(image.total_display_time);
                let file_name = Path::new(&image.filename).file_name().unwrap().to_string_lossy().into_owned();
                self.images_store.set(
                    &iter,
                    &[0, 1, 2, 3, 4, 5],
                    &[
                        &file_name,
                        &image.total_displays,
                        &image.total_skips,
                        &total_display_time_str,
                        &image.filename,
                        &image.total_display_time
                    ]
                );
            }
        }
    }

    fn load_image(&self) {
        if let Some((model, iter)) = self.image_selection.get_selected() {
            let value = model.get_value(&iter, 4);
            let file_path = value.get::<&str>().unwrap();
            let pixbuf = Pixbuf::new_from_file(file_path).expect("Couldn't load image");
            *self.buffer_pixbuf.borrow_mut() = Some(pixbuf);
            self.preview_image.queue_draw();
        }
    }

    fn rescale_image(&self, ctx: &cairo::Context) {
        if let Some(ref pixbuf) = *self.buffer_pixbuf.borrow() {
            let rect = self.preview_image.get_allocation();

            let (b_w, b_h) = (pixbuf.get_width(), pixbuf.get_height());
            let (r_w, r_h) = (rect.width, rect.height);

            let k_buf = b_h as f64 / b_w as f64;
            let k_rect = r_h as f64 / r_w as f64;

            let (n_w, n_h) = if k_buf < k_rect {
                (rect.width, (k_buf * rect.width as f64) as i32)
            } else {
                ((rect.height as f64 / k_buf) as i32, rect.height)
            };

            let (x, y) = ((r_w - n_w) / 2, (r_h - n_h) / 2);

            let scaled_pb = pixbuf.scale_simple(n_w, n_h, InterpType::Bilinear).unwrap();
            ctx.set_source_pixbuf(&scaled_pb, x as f64, y as f64);
            ctx.paint();
        }
    }
}

fn format_display_time(total_seconds: i64) -> String {
    let total_minutes = total_seconds / 60;
    let total_hours = total_minutes / 60;

    let hours = total_hours;
    let minutes = total_minutes % 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
