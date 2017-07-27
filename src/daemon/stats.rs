use std::path::Path;
use std::error::Error;
use std::ops::Add;

use diesel;
use diesel::*;
use diesel::sqlite::{Sqlite, SqliteConnection};
use diesel::types::ops as dtops;
use diesel::expression::operators as deoperators;
use diesel::expression::ops as deops;
use diesel::query_builder::update_statement::changeset::Changeset;
use diesel::expression::bound::Bound;

embed_migrations!();

mod schema {
    infer_schema!("stats.db");

    numeric_expr!(image_statistics::dsl::total_displays);
    numeric_expr!(image_statistics::dsl::total_skips);
    numeric_expr!(image_statistics::dsl::total_display_seconds);
}

mod model {
    use super::schema::*;

    #[derive(Queryable)]
    pub struct ImageStatistics {
        pub filename: String,
        pub total_displays: i64,
        pub total_skips: i64,
        pub total_display_time: i64,
    }

    #[derive(Insertable)]
    #[table_name="image_statistics"]
    pub struct NewImageStatistics<'a> {
        pub filename: &'a str,
    }
}

mod utils {
    use diesel::query_builder::*;
    use diesel::query_builder::insert_statement::Insert;
    use diesel::result::QueryResult;
    use diesel::sqlite::Sqlite;
    use diesel::expression::operators::Or;

    #[derive(Debug, Copy, Clone)]
    pub struct Ignore;

    impl QueryFragment<Sqlite> for Ignore {
        fn walk_ast(&self, mut out: AstPass<Sqlite>) -> QueryResult<()> {
            out.push_sql("IGNORE");
            Ok(())
        }
    }

    impl_query_id!(Ignore);

    pub fn insert_or_ignore<T: ?Sized>(records: &T) -> IncompleteInsertStatement<&T, Or<Insert, Ignore>> {
        IncompleteInsertStatement::new(records, Or::new(Insert, Ignore))
    }
}

pub type Result<T> = ::std::result::Result<T, Box<Error>>;  // unit for now

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
        self.update_number(file_name, self::schema::image_statistics::dsl::total_displays, n)
    }

    fn register_skips(&self, file_name: &str, n: i64) -> Result<()> {
        debug!("Registering skips for {}: {}", file_name, n);
        self.update_number(file_name, self::schema::image_statistics::dsl::total_skips, n)
    }

    fn register_display_time(&self, file_name: &str, time_sec: i64) -> Result<()> {
        debug!("Registering display time for {}: {}", file_name, time_sec);
        self.update_number(file_name, self::schema::image_statistics::dsl::total_display_seconds, time_sec)
    }

    fn update_number<F>(&self, file_name: &str, field: F, n: i64) -> Result<()>
        where F: Expression + Column<Table=self::schema::image_statistics::table> + Copy,
              F: AppearsOnTable<self::schema::image_statistics::table>,
              F: Add<i64, Output=deops::Add<F, Bound<diesel::types::BigInt, i64>>>,
              F::SqlType: dtops::Add<Output=F::SqlType>,
              deoperators::Eq<F, F::Output>: Changeset<Sqlite>
    {
        self.conn.transaction(|| {
            use self::schema::image_statistics::dsl::*;

            utils::insert_or_ignore(&model::NewImageStatistics { filename: file_name, })
                .into(image_statistics)
                .execute(&self.conn)?;

            diesel::update(image_statistics.filter(filename.eq(file_name)))
                .set(field.eq(field + n))
                .execute(&self.conn)?;

            Ok(())
        })
    }
}
