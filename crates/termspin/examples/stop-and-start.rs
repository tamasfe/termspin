use std::{io::stdout, thread, time::Duration};

use termspin::{spinner, Line, Loop};

fn main() {
    let task = Line::new(spinner::dots()).with_text("waiting ...").shared();

    let spin_loop = Loop::new(Duration::from_millis(100), task.clone());

    spin_loop.spawn_stream(stdout());

    thread::sleep(Duration::from_secs(2));

    task.lock().set_text("stopped.");

    spin_loop.stop();

    thread::sleep(Duration::from_secs(1));

    task.lock().set_text("waiting again ...");

    spin_loop.clear_stream(stdout()).unwrap();
    spin_loop.spawn_stream(stdout());

    thread::sleep(Duration::from_secs(2));

    println!();
}
