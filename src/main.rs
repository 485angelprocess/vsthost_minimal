extern crate vst;

mod midiin;

use std::env;
use std::path::Path;
use std::sync::{Arc, Mutex};

use vst::host::{Host, PluginLoader};
use vst::plugin::Plugin;

use midiin::MidiInputBuffer;

struct SampleHost;

impl Host for SampleHost {
    fn automate(&self, index: i32, value: f32) {
        println!("Parameter {} had its value changed to {}", index, value);
    }

    fn process_events(&self, events: &vst::api::Events) {
        //println!("Number of events, {}", events.num_events);
        /*let v = unsafe { std::slice::from_raw_parts(events.events[0], events.num_events as usize) };

        if v.len() > 0 {
            println!("{:#?}", v[0]._reserved);
        }*/

        /*for element in v.iter() {
            println!("{:#?}", element.event_type);
        }*/
        for event in events.events() {
            match event {
                vst::event::Event::Midi(vst::event::MidiEvent { data, .. }) => {
                    println!("Received midi data: {:#?}", data);
                }
                _ => {
                    println!("Received non midi event")
                }
            }
        }
    }
}

fn main() {
    let host = Arc::new(Mutex::new(SampleHost));

    //Take path as argument
    let args: Vec<String> = env::args().collect();

    //Pint help information (TODO: clean up)
    let helpstring = "Usage: vsthost_minimal PathToVST";
    if args.len() == 1 {
        println!("{}", helpstring);
        std::process::exit(1);
    }
    if &args[1] == "h" || &args[1] == "help" {
        println!("{}", helpstring);
        std::process::exit(1);
    }

    println!("Argument: {}, {}", &args[0], &args[1]);

    //Load plugin
    let path =
        Path::new("/Users/lizclaire/Samantha/vstdemo/SynthDemo.vst/Contents/MacOS/SynthDemo");

    let mut loader = PluginLoader::load(path, host.clone()).unwrap();
    let mut instance = loader.instance().unwrap();

    println!("Loaded {}", instance.get_info().name);

    instance.init();
    println!("Initialized instance!");

    let mut midiinbuffer = MidiInputBuffer::new();

    midiinbuffer.connect_midi_input();

    loop {
        let host = Arc::clone(&host);

        let host = host.lock().unwrap();

        //Obtain lock
        let (events, lock) = unsafe { midiinbuffer.get_events() };

        // Do all events processing here
        host.process_events(&events);

        //Drop lock
        drop(lock);

        unsafe { midiinbuffer.clear_midi_buffer() };
    }

    //println!("Closing instance...");
    // Not necessary as the instance is shut down when it goes out of scope anyway.
    // drop(instance);
}
