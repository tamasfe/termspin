use std::{io::stdout, thread, time::Duration};

use termspin::{spinner, Line, Loop};

fn main() {
    let task = Line::new(spinner::dots()).with_text("waiting ...").shared();

    let spin_loop = Loop::new(Duration::from_millis(100), task.clone());

    spin_loop.spawn_stream(stdout());

    thread::sleep(Duration::from_secs(2));

    task.lock().set_text("done.");

    drop(spin_loop);

    thread::sleep(Duration::from_secs(1));
    println!();
}
