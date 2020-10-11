use midir::{MidiOutput, MidiOutputPort};
use pasts::{prelude::*, CvarExec};
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use stick::{Event, Hub, Pad};

#[tokio::main]
async fn main() {
    let mut hub = Hub::new();
    let mut pads = Vec::<Pad>::new();

    let midi_out = MidiOutput::new("My Test Output").unwrap();

    // Get an output port (read from console if multiple are available)
    let out_ports = midi_out.ports();
    let out_port: &MidiOutputPort = match out_ports.len() {
        0 => return,
        1 => {
            println!(
                "Choosing the only available output port: {}",
                midi_out.port_name(&out_ports[0]).unwrap()
            );
            &out_ports[0]
        }
        _ => {
            println!("\nAvailable output ports:");
            for (i, p) in out_ports.iter().enumerate() {
                println!("{}: {}", i, midi_out.port_name(p).unwrap());
            }
            print!("Please select output port: ");
            stdout().flush().ok();
            let mut input = String::new();
            stdin().read_line(&mut input).ok();
            out_ports
                .get(input.trim().parse::<usize>().unwrap())
                .ok_or("invalid output port selected")
                .unwrap()
        }
    };

    println!("\nOpening connection");
    let mut conn_out = midi_out.connect(out_port, "midir-test").unwrap();

    sleep(Duration::from_millis(4 * 150));
    println!("Connection open. Listen!");

    let mut duration_bars: u64 = 2;

    const NOTE_ON_MSG: u8 = 0x90;
    const NOTE_OFF_MSG: u8 = 0x80;
    const VELOCITY: u8 = 0x64;
    const note: u8 = 63;

    'e: loop {
        match [hub.fut(), pads.select().fut()].select().await.1 {
            (_, Event::Connect(pad)) => {
                println!(
                    "Connected p{}, id: {:04X}_{:04X}_{:04X}_{:04X}, name: {}",
                    pads.len() + 1,
                    pad.id()[0],
                    pad.id()[1],
                    pad.id()[2],
                    pad.id()[3],
                    pad.name(),
                );
                pads.push(*pad);
            }
            (id, Event::Disconnect) => {
                println!("Disconnected p{}", id + 1);
                pads.swap_remove(id);
            }
            (id, Event::Home(true)) => {
                println!("p{} ended the session", id + 1);
                break 'e;
            }
            (id, event) => {
                println!("p{}: {}", id + 1, event);
                match event {
                    Event::ActionA(pressed) => {
                        if pressed {
                            let _ = conn_out.send(&[NOTE_ON_MSG, note, VELOCITY]);
                        } else {
                            let _ = conn_out.send(&[NOTE_OFF_MSG, note, VELOCITY]);
                        }
                    }
                    Event::ActionB(pressed) => {
                        pads[id].rumble(if pressed { 0.25 } else { 0.0 });
                    }
                    Event::BumperL(pressed) => {
                        duration_bars = if pressed { 1 } else { 2 };
                    }
                    _ => {}
                }
            }
        }
    }
}

fn play_note(conn_out: &mut midir::MidiOutputConnection, note: u8, duration: u64) {
    const NOTE_ON_MSG: u8 = 0x90;
    const NOTE_OFF_MSG: u8 = 0x80;
    const VELOCITY: u8 = 0x64;
    // We're ignoring errors in here
    let _ = conn_out.send(&[NOTE_ON_MSG, note, VELOCITY]);
    sleep(Duration::from_millis(duration * 150));
    let _ = conn_out.send(&[NOTE_OFF_MSG, note, VELOCITY]);
}
