use std::fs::File;
use simplelog::*;

pub trait Init {
    fn init_logger(&self);

    fn announce(&self);

    fn get_storage_location(&self) -> &'static str;
    fn get_logfile_location(&self) -> &'static str;

    fn get_command_listener_port(&self) -> String;
}

pub struct LocalEnvironment;
pub struct ServerEnvironment;

//TODO: implement this stuff using dotenv crate

impl Init for LocalEnvironment {
    fn init_logger(&self) {
        let log_file = File::create(self.get_logfile_location())
            .expect("Could not create log file");

        CombinedLogger::init(vec![
            SimpleLogger::new(LevelFilter::Debug, Config::default()),
            WriteLogger::new(LevelFilter::Trace, Config::default(), log_file),
        ]).expect("Could not initialise combined logger");
    }

    fn announce(&self) {
        debug!("Starting in local enviroment");
    }

    fn get_storage_location(&self) -> &'static str {
        unimplemented!()
    }

    fn get_logfile_location(&self) -> &'static str {
        "./dengbot.log"
    }

    fn get_command_listener_port(&self) -> String {
        // TODO
        String::from("60400")
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

    fn announce(&self) {
        debug!("Starting in server enviroment");

        if let Ok(port) = ::std::env::var(::HEROKU_PORT_ENV_VAR) {
            debug!("Heroku has assigned port {}", port);
        }
    }

    fn get_storage_location(&self) -> &'static str {
        unimplemented!()
    }

    fn get_logfile_location(&self) -> &'static str {
        "./dengbot.log"
    }

    fn get_command_listener_port(&self) -> String {
        ::std::env::var(::HEROKU_PORT_ENV_VAR)
            .expect("Could not find port for command listener binding")
    }
}
