use std::fs::File;
use simplelog::*;
use deng::Deng;

pub trait Init {
    fn init_logger(&self);
    fn init_storage(&self) -> Vec<Deng>;

    fn announce(&self);

    fn get_storage_location(&self) -> &'static str;
    fn get_logfile_location(&self) -> &'static str;
}

pub struct LocalEnvironment;
pub struct ServerEnvironment;

impl Init for LocalEnvironment {
    fn init_logger(&self) {
        let log_file = File::create(self.get_logfile_location())
            .expect("Could not create log file");

        CombinedLogger::init(vec![
            SimpleLogger::new(LevelFilter::Debug, Config::default()),
            WriteLogger::new(LevelFilter::Trace, Config::default(), log_file),
        ]).expect("Could not initialise combined logger");
    }

    fn init_storage(&self) -> Vec<Deng> {
        ::dengstorage::read(self.get_storage_location())
    }

    fn announce(&self) {
        debug!("Starting in local enviroment");
    }

    fn get_storage_location(&self) -> &'static str {
        "./dengs"
    }

    fn get_logfile_location(&self) -> &'static str {
        "./dengbot.log"
    }
}

impl Init for ServerEnvironment {
    fn init_logger(&self) {
        let log_file = File::create(self.get_logfile_location())
            .expect("Could not create log file");

        CombinedLogger::init(vec![
            TermLogger::new(LevelFilter::Debug, Config::default())
                .expect("Could not initialise terminal logger"),
            WriteLogger::new(LevelFilter::Trace, Config::default(), log_file),
        ]).expect("Could not initialise combined logger");
    }

    fn init_storage(&self) -> Vec<Deng> {
        ::dengstorage::read(self.get_storage_location())
    }

    fn announce(&self) {
        debug!("Starting in server enviroment");

        if let Ok(port) = ::std::env::var(::HEROKU_PORT_ENV_VAR) {
            debug!("Heroku has assigned port {}", port);
        }
    }

    fn get_storage_location(&self) -> &'static str {
        "./dengs"
    }

    fn get_logfile_location(&self) -> &'static str {
        "./dengbot.log"
    }
}
