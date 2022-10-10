use std::{io::stdout, thread, time::Duration};

use termspin::{spinner, Group, Line, Loop, SharedFrames};

fn main() {
    let dots = spinner::from_iter(["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

    let main_group = Group::new();

    let main_group = SharedFrames::new(main_group);

    let spin_loop = Loop::new(Duration::from_millis(100), main_group.clone());
    let l = spin_loop.clone();
    thread::spawn(move || l.run_stream(stdout()));

    let main_task = Line::new(dots.clone())
        .with_text("executing main task...")
        .shared();
    main_group.lock().push(main_task.clone());

    let subtask_group = Group::new().with_indent(1).shared();

    let subtasks = (0..5)
        .map(|i| {
            Line::new(dots.clone())
                .with_text(&format!("executing subtask {i}..."))
                .shared()
        })
        .collect::<Vec<_>>();

    subtask_group.lock().extend(subtasks.iter().cloned());

    main_group.lock().push(subtask_group.clone());

    thread::sleep(Duration::from_secs(2));

    for (i, subtask) in subtasks.into_iter().enumerate() {
        subtask
            .lock()
            .set_spinner_visible(false)
            .set_text(&format!("✓ subtask {i} complete."));
        thread::sleep(Duration::from_millis(500));
    }

    subtask_group
        .lock()
        .push(Line::new(spinner::empty()).with_text("✓ this task was added after."));

    main_task
        .lock()
        .set_spinner_visible(false)
        .set_text("✓ first main task done.");

    thread::sleep(Duration::from_secs(1));

    let last_task = Line::new(dots.clone())
        .with_text("almost ready, do not turn off your computer...")
        .shared();

    main_group.lock().push(last_task.clone());

    thread::sleep(Duration::from_secs(5));

    last_task
        .lock()
        .set_spinner_visible(false)
        .set_text("x fatal error.");

    spin_loop.stop();

    thread::sleep(Duration::from_secs(2));

    spin_loop.clear_stream(stdout()).unwrap();
}
