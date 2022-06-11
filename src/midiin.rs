extern crate lazy_static;
extern crate midir;
extern crate vst;

use lazy_static::lazy_static;
use midir::{Ignore, MidiInput};
use std::error::Error;
use std::mem;
use std::sync::{Arc, Mutex, RwLock, RwLockWriteGuard};
use vst::api::{Event, Events};

use std::io::{stdin, stdout, Write};

lazy_static! {
    static ref MIDI_IN_EVENTS: RwLock<Vec<Event>> = RwLock::new(Vec::new());
}

pub struct MidiInputBuffer<'a> {
    midi_in_conn: Option<Arc<Mutex<midir::MidiInputConnection<()>>>>,
    write_lock: Option<RwLockWriteGuard<'a, Vec<Event>>>,
}

fn pad_zeroes<const A: usize, const B: usize>(arr: &[u8; A]) -> [u8; B] {
    assert!(B >= A); //just for a nicer error message, adding #[track_caller] to the function may also be desirable
    let mut b = [0; B];
    b[..A].copy_from_slice(arr);
    b
}

unsafe fn capture_midi_in(_stamp: u64, &message: &[u8; 3]) {
    //println!("Received message: {:#?}", message);
    let midi_event: vst::api::MidiEvent = vst::api::MidiEvent {
        event_type: vst::api::EventType::Midi,
        byte_size: mem::size_of::<vst::api::MidiEvent>() as i32,
        delta_frames: 0,
        flags: vst::api::MidiEventFlags::empty().bits(),
        note_length: 0,
        note_offset: 0,
        midi_data: message,
        _midi_reserved: 0,
        detune: 0,
        note_off_velocity: 0,
        _reserved1: 0,
        _reserved2: 0,
    };

    let event: &vst::api::Event = std::mem::transmute(&midi_event);

    // Add to midi buffer
    let mut events = MIDI_IN_EVENTS.write().unwrap();
    events.push(*event);

    /*println!(
        "Number of events {}, capacity, {}",
        events.len(),
        events.capacity()
    );*/
    /*let processed_event = vst::event::Event::from_raw_event(&midi_event);
    match processed_event {
        vst::event::Event::Midi(vst::event::MidiEvent { data, .. }) => {
            assert_eq!(data[0], message[0])
        }
        _ => {}
    }*/
    let event = events.as_ptr();

    match (*event).event_type {
        vst::api::EventType::Midi => {
            let midi_event: &vst::api::MidiEvent = std::mem::transmute(event);
            assert_eq!(midi_event.midi_data[0], message[0]);
        }
        _ => {}
    }

    drop(events);
}

impl MidiInputBuffer<'_> {
    pub fn new() -> Self {
        MidiInputBuffer {
            midi_in_conn: None,
            write_lock: None,
        }
    }

    pub unsafe fn get_events<'a>(
        &mut self,
    ) -> (vst::api::Events, RwLockWriteGuard<'a, Vec<Event>>) {
        /* Get active events in buffer */
        /*let events: [*mut vst::api::Event; 2] = match (MIDI_IN_EVENTS.len()) {
            0 => [std::ptr::null_mut(), std::ptr::null_mut()],
            1 => [&mut MIDI_IN_EVENTS[0], std::ptr::null_mut()],
            _ => [&mut MIDI_IN_EVENTS[0], &mut MIDI_IN_EVENTS[1]],
        };*/

        //MIDI_IN_EVENTS.truncate(2);
        let mut write_lock: RwLockWriteGuard<'a, Vec<Event>> = MIDI_IN_EVENTS.write().unwrap();

        let ptr = write_lock.as_mut_ptr();
        let len = write_lock.len();

        let events: [*mut vst::api::Event; 2] = match len {
            0 => [
                std::ptr::NonNull::<vst::api::Event>::dangling().as_ptr(),
                std::ptr::NonNull::<vst::api::Event>::dangling().as_ptr(),
            ],
            _ => [
                ptr,
                std::ptr::NonNull::<vst::api::Event>::dangling().as_ptr(),
            ],
        };
        (
            Events {
                num_events: len as i32,
                _reserved: 0,
                events: events,
            },
            write_lock,
        )
    }

    pub unsafe fn clear_midi_buffer(&mut self) {
        drop(self.write_lock.as_ref());

        let mut events = MIDI_IN_EVENTS.write().unwrap();
        events.truncate(0);
        events.shrink_to_fit();
        drop(events);
    }

    pub fn connect_midi_input(&mut self) -> Result<(), Box<dyn Error>> {
        /* Set up midi connection
            Initialize call back to capture midi in events
        */
        let mut midi_in = MidiInput::new("Rust VST host")?;

        //TODO: look into
        midi_in.ignore(Ignore::None);

        let in_ports = midi_in.ports();
        //Select port
        let in_port = match in_ports.len() {
            0 => return Err("No input ports available".into()),
            1 => {
                println!("Using port: {}", midi_in.port_name(&in_ports[0]).unwrap());
                &in_ports[0]
            }
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
                in_ports
                    .get(input.trim().parse::<usize>()?)
                    .ok_or("invalid port selected")?
            }
        };

        println!("Opening connection");
        let in_port_name = midi_in.port_name(in_port)?;

        self.midi_in_conn = Some(Arc::new(Mutex::new(midi_in.connect(
            in_port,
            "midir-read-input",
            move |stamp, message, _| unsafe {
                capture_midi_in(stamp, message.try_into().expect("Message is wrong length"))
            },
            (),
        )?)));

        Ok(())
    }
}
