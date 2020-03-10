use chrono::{Local, Timelike};
use cursive::traits::*;
use cursive::views::{Dialog, DummyView, LinearLayout, ListView};
use cursive::Cursive;
use rodio::Source;
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct ClockData {
    first_num: Option<u8>,
    second_alarm: bool,
    alarms: Vec<(String, bool)>,
}

fn main() {
    ::std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}

fn run_app() -> Result<(), std::boxed::Box<(dyn std::any::Any + std::marker::Send + 'static)>> {
    let mut siv = Cursive::crossterm().unwrap();
    let data: ClockData = ClockData {
        first_num: None,
        second_alarm: false,
        alarms: vec![],
    };
    siv.set_user_data(data);
    let running = Arc::new(AtomicBool::new(true));

    let col1 = ListView::new().with_id("col1").fixed_size((5, 5));

    let col2 = ListView::new().with_id("col2").fixed_size((3, 5));

    siv.add_layer(
        Dialog::around(LinearLayout::horizontal().child(col1).child(col2))
            .title("Alarm")
            .with_id("header"),
    );

    siv.add_global_callback('q', |s| s.quit());
    siv.add_global_callback('*', toggle_sound);
    siv.add_global_callback('-', clear_alarms);

    for i in 0u8..=9 {
        let i_char: char = std::char::from_digit(i as u32, 10).unwrap();
        siv.add_global_callback(i_char, move |s| handle_numbers(s, i));
    }

    let sink = siv.cb_sink().clone();
    let running_clone = running.clone();

    let clock_thread = thread::spawn(move || {
        let last_played = Arc::new(AtomicU32::new(60));
        let mut last_second: u32 = 60;
        let mut last_hour: u32 = 30;
        while running_clone.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));
            //shadowing
            if last_hour != Local::now().hour() {
                last_hour = Local::now().hour();
                last_played.store(60, Ordering::Relaxed);
            }
            let last_played = last_played.clone();
            if last_second != Local::now().second() {
                sink.send(Box::new(|s: &mut Cursive| alarm_it(s, last_played)))
                    .expect("Could not send alarm sound");
                sink.send(Box::new(update_time))
                    .expect("Could not update time header");
                last_second = Local::now().second();
            }
        }
    });

    siv.run();

    //End the clock_thread when the main thread ends
    running.store(false, Ordering::Relaxed);

    //This ensures every thread is finished when the program ends
    clock_thread.join()
}

//pub fn alarm_it(s: &mut Cursive, alarm_sink: Arc<Sink>, last_played: Arc<AtomicU32>, last_second: Arc<AtomicU32>) {
pub fn alarm_it(s: &mut Cursive, last_played: Arc<AtomicU32>) {
    if last_played.load(Ordering::Relaxed) == Local::now().minute() {
        return;
    }
    let data: &mut ClockData = s.user_data().unwrap();
    let alarms: Vec<(u32, &bool)> = data
        .alarms
        .iter()
        .map(|(x, y)| (x.parse::<u32>().unwrap(), y))
        .collect();
    //check if the minute of the time is the minute of one alarm time
    for (alarm, toggle) in alarms {
        let curr_min = Local::now().minute();
        if alarm == curr_min && last_played.load(Ordering::Relaxed) != curr_min {
            let file_name = if *toggle {
                "soundalarm2.wav"
            } else {
                "soundalarm.wav"
            };
            play(file_name);
            last_played.store(curr_min, Ordering::Relaxed);
            return;
        }
    }
    return;
}

pub fn update_time(s: &mut Cursive) -> () {
    s.call_on_id("header", |dialog: &mut Dialog| {
        dialog.set_title(Local::now().format("%H:%M:%S").to_string())
    });
}

pub fn play(file_name: &str) -> () {
    let device = rodio::default_output_device().expect("Could not find sound device");
    let file = File::open(file_name).expect("Could not find sound file");
    let source = rodio::Decoder::new(BufReader::new(file)).expect("Could not create sound decoder");
    rodio::play_raw(&device, source.convert_samples());
}

pub fn sync() {
    let start = chrono::Local::now().nanosecond() as u64;
    let delay = std::time::Duration::from_nanos(1_000_000_000 - start);
    std::thread::sleep(delay);
}

fn toggle_sound(s: &mut Cursive) {
    let mut data: ClockData = s.take_user_data().unwrap();
    data.second_alarm = !data.second_alarm;
    s.set_user_data(data);
}

fn clear_alarms(s: &mut Cursive) {
    let mut data: ClockData = s.take_user_data().unwrap();
    data.alarms.clear();
    s.set_user_data(data);
    for c in vec!["col1", "col2"] {
        s.call_on_id(c, |col: &mut ListView| col.clear());
    }
}

fn handle_numbers(s: &mut Cursive, num: u8) {
    let mut data: ClockData = s.take_user_data().unwrap();
    let alarms: Vec<&str> = data.alarms.iter().map(|(x, _)| x.as_str()).collect();
    let first_num: Option<u8> = data.first_num;
    if first_num.is_none() && num < 6 {
        data.first_num = Some(num);
    } else if first_num.is_some() {
        let mut label = String::from(first_num.unwrap_or_default().to_string());
        label.push_str(num.to_string().as_str());
        let mut recreate = false;
        if alarms.contains(&label.as_str()) {
            let index = alarms.iter().position(|x| *x == label).unwrap();
            data.alarms.remove(index);
            recreate = true;
        } else {
            if data.alarms.len() < 10 {
                data.alarms.push((label, data.second_alarm));
                data.second_alarm = false;
                recreate = true;
            }
        }
        if recreate {
            data.alarms.sort();
            for c in vec!["col1", "col2"] {
                s.call_on_id(c, |col: &mut ListView| col.clear());
            }
            for (i, (alarm, toggle)) in data.alarms.iter().enumerate() {
                let col = if i < 5 { "col1" } else { "col2" };
                s.call_on_id(col, |view: &mut ListView| {
                    let label: String = if *toggle { "*" } else { "" }.to_owned() + alarm.as_str();
                    view.add_child(label.as_str(), DummyView);
                });
            }
        }
        data.first_num = None;
    }
    s.set_user_data(data);
}
