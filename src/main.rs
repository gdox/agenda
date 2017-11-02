extern crate getopts;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate rustbreak;
extern crate chrono;

const CONFIG_LOCATION : &'static str = "/etc/agenda/agenda.conf";

mod parse;

use rustbreak::Database;
use getopts::Options;

use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::io::Read;
use std::path::Path;
use std::process::exit;

type DateTime = chrono::DateTime<chrono::Utc>;

use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
struct Config {
    database : String,
}

#[derive(Serialize, Deserialize)]
struct EventLog {
    events : HashSet<usize>,
    highest : usize,
}

impl EventLog {
    pub fn new() -> EventLog {
        EventLog {
            events : HashSet::new(),
            highest : 0,
        }
    }

    pub fn get_events(&self) -> &HashSet<usize> {
        &self.events
    }

    pub fn add_event(&mut self) -> usize {
        self.events.insert(self.highest);
        let res = self.highest;
        self.highest += 1;
        res
    }

    pub fn remove_event(&mut self, event : usize) -> bool{
        self.events.remove(&event)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    index : usize,
    created : DateTime,
    description : String,
    deadline : Option<DateTime>,
}

pub fn write_datetime(datetime : Option<DateTime>) -> String {
    datetime.as_ref().map(DateTime::to_rfc2822).unwrap_or_else(|| "Unspecified".to_string())
}

struct WriteEvent<'a>(pub &'a Event);
struct WriteShortEvent<'a>(pub &'a Event);

impl<'a> fmt::Display for WriteEvent<'a> {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (self.0).write(f)
    }
}

impl<'a> fmt::Display for WriteShortEvent<'a> {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (self.0).write_short(f)
    }
}

impl Event {
    pub fn new(index : usize, description : String, created : DateTime, deadline : Option<DateTime>) -> Event {
        Event {
            index : index,
            description : description,
            created : created,
            deadline : deadline,
        }
    }
    pub fn write_short(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Event {}: {} (by {})", self.index, self.description, write_datetime(self.deadline))
    }
    pub fn write(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Event {} (from {}):\n\t{}\n\tDeadline: {}",
            self.index,
            self.created,
            self.description,
            write_datetime(self.deadline))
    }
    pub fn get_event_tag(index : usize) -> String {
        format!("event_{}", index)
    }

    pub fn get_tag(&self) -> String {
        Event::get_event_tag(self.index)
    }
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(e) => {let _ = writeln!(io::stderr(), "An error has occurred: {} ({:?})", e, e); exit(2);}
    }
}

fn run() -> Result<(), Box<Error>> {
    let mut options = Options::new();
    options.optopt("d", "date", "Provide a deadline", "DATE");
    options.optflag("h", "help", "Show this error");
    options.optflag("s", "short", "Short display mode");
    options.optflag("t", "sort", "Sort on deadline instead of creation time");

    let matches = options.parse(&env::args().collect::<Vec<_>>()[1..])?;
    if matches.opt_present("h") {
        println!("{}", options.usage("Provides an agenda utility.\nUsage : \n\tagenda list [options]\n\tagenda add <message> [options]\n\tagenda delete <event id>"));
        return Ok(());
    }

    let config : Config =  {
        let config_filename = CONFIG_LOCATION;
        let mut string = String::new();
        let mut file = fs::File::open(config_filename).map_err(|e| {let _ = writeln!(io::stderr(), "File: {}", config_filename); e})?;
        file.read_to_string(&mut string)?;
        toml::from_str(&string)?
    };

    let db = Database::open(Path::new(&config.database)).map_err(|e| {let _ = writeln!(io::stderr(), "File: {}", config.database); e})?;
    let mut event_log : EventLog = db.retrieve("log").ok().unwrap_or_else(EventLog::new);
    match matches.free.get(0).map(|e| e.as_ref()) {
        Some("list") => {
            let mut events : Vec<Event> = event_log.get_events()
                        .iter()
                        .map(|&u| Event::get_event_tag(u))
                        .map(|s| db.retrieve::<Event, _>(&s))
                        .filter_map(Result::ok)
                        .collect();
            if matches.opt_present("t") {
                events.sort_by(|a, b|
                {
                    use std::cmp::Ordering;
                    match (a.deadline, b.deadline) {
                        (Some(ref x), Some(ref y)) => x.cmp(&y),
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
                        (None, None) => a.index.cmp(&b.index),
                    }
                })
            } else {
                events.sort_by (|a, b| a.index.cmp(&b.index))
            }
            if matches.opt_present("s") {
                for event in events {
                    println!("{}", WriteShortEvent(&event));
                }
            } else {
                for event in events {
                    println!("{}", WriteEvent(&event));
                }
            }
        },
        Some("add") => {
            let deadline = matches.opt_str("d").map(|m| parse::parse_date(&m));
            let deadline = match deadline {
                None => Ok(None),
                Some(Ok(x)) => Ok(Some(x)),
                Some(Err(e)) => Err(e),
            }?;
            let message = matches.free[1..].join(" ");
            let event_id = event_log.add_event();
            let now = chrono::Utc::now();
            let event = Event::new(event_id, message, now, deadline);
            println!("Added:\n{}", WriteEvent(&event));
            db.insert(&event.get_tag(), event)?;
            db.insert("log", event_log)?;
            db.flush()?;
        },
        Some("delete") => {
            let event_id : usize = matches.free.get(1).ok_or("No element specified to delete!")?.parse()?;
            event_log.remove_event(event_id);
            let event : Event = db.retrieve(&Event::get_event_tag(event_id))?;
            db.delete(&Event::get_event_tag(event_id))?;
            db.insert("log", event_log)?;
            db.flush()?;
            println!("Deleted:\n{}", WriteEvent(&event));
        },
        _ => {
            println!("{}", options.short_usage(&env::args().next().unwrap())); exit(3);
        }
    }

    Ok(())
}
