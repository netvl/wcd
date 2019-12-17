use std::path::Path;
use std::error::Error;

use diesel;
use diesel::*;
use diesel::sqlite::SqliteConnection;

embed_migrations!();

pub mod schema {
    table! {
        image_statistics (filename) {
            filename -> Text,
            total_displays -> BigInt,
            total_skips -> BigInt,
            total_display_seconds -> BigInt,
        }
    }
}

pub mod model {
    use crate::common::grpc::wcd;
    use super::schema::*;

    #[derive(Queryable)]
    pub struct ImageStatistics {
        pub filename: String,
        pub total_displays: i64,
        pub total_skips: i64,
        pub total_display_time: i64,
    }

    impl Into<wcd::ImageStatsInfo> for ImageStatistics {
        fn into(self) -> wcd::ImageStatsInfo {
            let mut proto = wcd::ImageStatsInfo::new();
            proto.set_filename(self.filename);
            proto.set_total_displays(self.total_displays);
            proto.set_total_skips(self.total_skips);
            proto.set_total_display_time(self.total_display_time);
            proto
        }
    }

    #[derive(Insertable)]
    #[table_name="image_statistics"]
    pub struct NewImageStatistics<'a> {
        pub filename: &'a str,
    }
}

pub type Result<T> = ::std::result::Result<T, Box<dyn Error>>;  // unit for now

#[derive(Clone)]
pub struct Stats {
    daemon: super::Daemon,
}

impl Stats {
    pub fn new(daemon: super::Daemon) -> Stats {
        Stats { daemon, }
    }

    pub fn register_displays(&self, file_name: &str, n: i64) -> Result<()> {
        if let Some(ref stats) = self.daemon.state.lock().stats {
            stats.borrow().register_displays(file_name, n)
        } else {
            Ok(())
        }
    }

    pub fn register_skips(&self, file_name: &str, n: i64) -> Result<()> {
        if let Some(ref stats) = self.daemon.state.lock().stats {
            stats.borrow().register_skips(file_name, n)
        } else {
            Ok(())
        }
    }

    pub fn register_display_time(&self, file_name: &str, time_sec: i64) -> Result<()> {
        if let Some(ref stats) = self.daemon.state.lock().stats {
            stats.borrow().register_display_time(file_name, time_sec)
        } else {
            Ok(())
        }
    }

    pub fn load(&self) -> Result<Vec<model::ImageStatistics>> {
        if let Some(ref stats) = self.daemon.state.lock().stats {
            stats.borrow().load()
        } else {
            Ok(Vec::new())
        }
    }
}

pub struct State {
    conn: SqliteConnection,
}

impl State {
    pub fn new(path: &Path) -> Result<State> {
        info!("Establishing connection to an SQLite database: {}", path.display());

        #[derive(Debug)]
        struct InvalidStatsDbDir;
        impl ::std::fmt::Display for InvalidStatsDbDir {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.write_str(self.description())
            }
        }
        impl ::std::error::Error for InvalidStatsDbDir {
            fn description(&self) -> &str {
                "Path to the statistics database is invalid"
            }
        }

        let db_dir = match path.parent() {
            Some(dir) => dir,
            None => return Err(InvalidStatsDbDir.into())
        };

        if !db_dir.exists() {
            ::std::fs::create_dir_all(db_dir)?;
            info!("Created directory: {}", db_dir.display());
        }
        
        let conn = SqliteConnection::establish(&path.as_os_str().to_string_lossy())?;

        info!("Running migrations");
        embedded_migrations::run(&conn)?;
        
        Ok(State { conn, })
    }

    fn register_displays(&self, file_name: &str, n: i64) -> Result<()> {
        debug!("Registering displays for {}: {}", file_name, n);
        let field = self::schema::image_statistics::dsl::total_displays;
        self.conn.transaction(|| {
            use self::schema::image_statistics::dsl::*;

            diesel::insert_or_ignore_into(image_statistics)
                .values(&model::NewImageStatistics { filename: file_name, })
                .execute(&self.conn)?;

            diesel::update(image_statistics.filter(filename.eq(file_name)))
                .set(field.eq(field + n))
                .execute(&self.conn)?;

            Ok(())
        })
    }

    fn register_skips(&self, file_name: &str, n: i64) -> Result<()> {
        debug!("Registering skips for {}: {}", file_name, n);
        let field = self::schema::image_statistics::dsl::total_skips;
        self.conn.transaction(|| {
            use self::schema::image_statistics::dsl::*;

            diesel::insert_or_ignore_into(image_statistics)
                .values(&model::NewImageStatistics { filename: file_name, })
                .execute(&self.conn)?;

            diesel::update(image_statistics.filter(filename.eq(file_name)))
                .set(field.eq(field + n))
                .execute(&self.conn)?;

            Ok(())
        })
    }

    fn register_display_time(&self, file_name: &str, time_sec: i64) -> Result<()> {
        debug!("Registering display time for {}: {}", file_name, time_sec);
        let field = self::schema::image_statistics::dsl::total_display_seconds;
        self.conn.transaction(|| {
            use self::schema::image_statistics::dsl::*;

            diesel::insert_or_ignore_into(image_statistics)
                .values(&model::NewImageStatistics { filename: file_name, })
                .execute(&self.conn)?;

            diesel::update(image_statistics.filter(filename.eq(file_name)))
                .set(field.eq(field + time_sec))
                .execute(&self.conn)?;

            Ok(())
        })
    }

    fn load(&self) -> Result<Vec<model::ImageStatistics>> {
        use self::schema::image_statistics::dsl::*;
        
        Ok(image_statistics
            .order(filename.asc())
            .load::<model::ImageStatistics>(&self.conn)?)
    }
}
