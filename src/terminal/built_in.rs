use crate::{println, print};
use crate::interrupts_handler::read_timer_seconds;


pub fn help_builtin () {
    println!("Available functions:");
    println!();
    println!("- help: Prints all available functions, takes no argument");
    print!("- uptime: Gives the uptime of the machine");
}

pub fn uptime_builtin() {
    let total = read_timer_seconds();
    let hours = total / 3600;
    let mins = (total / 60) % 60;
    let secs = total % 60;

    print!("Uptime =");
    if hours > 0 { print!(" {} hour{}", hours, plural(hours)); }
    if mins > 0  { print!(" {} minute{}", mins, plural(mins)); }
    print!(" {} second{}", secs, plural(secs));
}

fn plural(n: u64) -> &'static str {
    if n == 1 { "" } else { "s" }
}