extern crate vst;
extern crate midir;

use std::sync::{Arc, Mutex};
use std::path::Path;
use std::env;
use std::error::Error;

use vst::host::{Host, PluginLoader};
use vst::plugin::Plugin;
use vst::prelude::Events;

use midir::{MidiInput, MidiInputConnection, Ignore};

struct SampleHost {
    midi_in_conn: midir::MidiInputConnection,
    midi_in_events: Events,
}

pub enum EventsFlag {
    message = 0,
    exit = 1,
}

impl Host for SampleHost {
    fn automate(&self, index: i32, value: f32) {
        println!("Parameter {} had its value changed to {}", index, value);
    }

    fn process_events(&self, events: &Events) -> EventsFlag {
        for event in events.events() {
            if let vst::event::Event::Midi(midi) = event {
                println!("Received event: {}", event.data);
            }
        }
        EventsFlag::message
    }
}

trait HostHelper: Host{
    fn capture_midi_in(&self, stamp: u64, message: u8);
    fn connect_midi_input() -> Result<(), Box<dyn  Error>>;
}

impl<T> HostHelper for T where T: Host{
    fn capture_midi_in(&self, stamp: u64, message: u8) {
        let event = vst::event::MidiEvent{
            data: message,
        };
        // Add to midi buffer
        self.midi_in_events.push(event);
    }

    fn connect_midi_input() -> Result<(), Box<dyn  Error>> {
        /* Set up midi connection
            Initialize call back to capture midi in events
        */

        let midi_in = MidiInput::new("Rust VST host")?;

        //TODO: look into
        midi_in.ignore(Ignore::None);

        let in_ports = self.midi_in.ports();
        //Select port
        let in_ports = match in_ports.len() {
            0 => return Err("No input ports available".into()),
            1 => {
                println!("Using port: {}", self.midi_in.port_name(&in_ports[0]).unwrap());
                &in_ports[0]
            },
            _ => {
                println!("\nAvailable input ports:");
                // List available ports and indexes
                for (i, p) in in_ports.iter().enumerate() {
                    println!("{}: {}", i, midi_in.port_name(p).unwrap());
                }
                print!("Select input port: ");
                stdout().flush()?;
                let mut input = String::new();
                stdin().read_line(&mut input)?;
                in_port.get(input.trim().parse::<usize>()?)
                    .ok_or("invalid port selected")?
            }
        };

        println!("Opening connection");
        let in_port_name = midi_in.port_name(in_port)?;

        self.midi_in_conn = midi_in.connect(in_port, "rusthost-read-input", move |stamp, message, _| {
            self.capture_midi_in(stamp, message);
        }, ())?;
    }
}


fn main() {
    let host = Arc::new(Mutex::new(SampleHost));

    //Take path as argument
    let args: Vec<String> = env::args().collect();


    //Pint help information (TODO: clean up)
    let helpstring = "Usage: vsthost_minimal PathToVST";
    if args.len() == 1{
        println!("{}", helpstring);
        std::process::exit(1);
    }
    if &args[1] == "h" || &args[1] == "help"{
        println!("{}", helpstring);
        std::process::exit(1);
    }

    println!("Argument: {}, {}", &args[0], &args[1]);

    //Load plugin
    let path = Path::new("/Users/lizclaire/Samantha/vstdemo/SynthDemo.vst/Contents/MacOS/SynthDemo");

    let mut loader = PluginLoader::load(path, host.clone()).unwrap();
    let mut instance = loader.instance().unwrap();

    println!("Loaded {}", instance.get_info().name);

    instance.init();
    println!("Initialized instance!");

    while(true){

        plugin.process_events()

    }

    println!("Closing instance...");
    // Not necessary as the instance is shut down when it goes out of scope anyway.
    // drop(instance);
}
