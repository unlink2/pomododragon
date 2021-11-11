use clap::Parser;
use spinners::{Spinner, Spinners};
use std::thread;
use std::time::Duration;

use pomododragon::{InstantTimer, Pomo, PomoState, SimplePomoBuilder, SimpleTask, Timer};

#[derive(Parser)]
#[clap(version = "0.1.0", author = "Lukas Krickl <lukas@krickl.dev>")]
struct Opts {
    #[clap(short, long, default_value = "300")]
    break_time_secs: u64,
    #[clap(short, long, default_value = "1500")]
    work_time_secs: u64,
    #[clap(short, long, default_value = "1800")]
    rest_time_secs: u64,

    #[clap(short, long, default_value = "10")]
    poll_millis: u64,

    #[clap(short, long, default_value = "4")]
    until_break: usize,
    #[clap(short, long, default_value = "6")]
    total: usize,

    tasks: Vec<String>,
}

fn main() {
    let stdout = std::io::stdout();
    let opts: Opts = Opts::parse();

    let mut tasks = vec![];

    for s in opts.tasks {
        tasks.push(SimpleTask::new(s));
    }

    let mut pomo = SimplePomoBuilder::<SimpleTask, InstantTimer>::default()
        .break_timer(InstantTimer::new(Duration::from_secs(opts.break_time_secs)))
        .work_timer(InstantTimer::new(Duration::from_secs(opts.work_time_secs)))
        .rest_timer(InstantTimer::new(Duration::from_secs(opts.rest_time_secs)))
        .cycles_until_rest(opts.until_break)
        .total_cycles(opts.total)
        .tasks(tasks)
        .build()
        .expect("Unable to build pomo");

    let sp = if !termion::is_tty(&stdout) {
        None
    } else {
        // println!("(q) to exit; (p) to pause");
        Some(Spinner::new(&Spinners::Dots, "".into()))
    };

    while !pomo.is_completed() {
        pomo.update().expect("Error while processing timer");

        let message = if pomo.is_paused() {
            "Paused".into()
        } else {
            let state = match pomo.state() {
                PomoState::NotStarted => "Not Started",
                PomoState::Working => "Working",
                PomoState::Break => "Break",
                PomoState::Resting => "Resting",
                PomoState::Paused => "Paused",
                PomoState::Completed => "Completed",
            };

            let task = match pomo.task() {
                Some(task) => task.to_string(),
                None => "".into(),
            };

            let timer = match pomo.timer() {
                Some(timer) => timer.elapsed().unwrap_or_else(|| Duration::from_secs(0)),
                None => Duration::from_secs(0),
            };

            let mins = timer.as_secs() / 60;
            let secs = timer.as_secs() - mins * 60;
            format!("[{}] [{}] [{:02}:{:02}]", state, task, mins, secs)
        };

        if let Some(sp) = &sp {
            sp.message(message);
        } else {
            println!("{}", message);
        }

        thread::sleep(Duration::from_millis(opts.poll_millis));
    }
    if let Some(sp) = sp {
        sp.stop();
    }
}
