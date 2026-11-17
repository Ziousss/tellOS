use core::str::SplitAsciiWhitespace;

use crate::vga_buffer::clear_screen;
use crate::{println, print};
use crate::interrupts_handler::read_timer_seconds;


pub fn help_builtin () {
    println!("Available functions:");
    println!();
    println!("- clear: Clears the screen");
    println!("- echo: Prints to the terminal whatever it is given. Takes one argument");
    println!("- help: Prints all available functions, takes no argument");
    println!("- uptime: Gives the uptime of the machine");
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
    println!();
}

fn plural(n: u64) -> &'static str {
    if n == 1 { "" } else { "s" }
}

pub fn echo_builtin(mut words: SplitAsciiWhitespace<'_>) {
    let mut newline = true; 


    let _ = words.next();
    if let Some(first_arg) = words.next() {
        if first_arg == "-n" {
            newline = false;
        } else {
            print!("{}", first_arg);
            print!(" ");
        }
    }

    let mut to_print;

    match words.next() {
        Some(c) => to_print = c,
        None => to_print = "",
    }

    loop {
        print!("{}", to_print);

        match words.next() {
            Some(c) => to_print = c,
            None => to_print = "",
        }

        if to_print != "" {
            print!(" ");
            continue;
        } else {
            break;
        }
    }    

    if newline {
        println!();
    }
}

pub fn clear_builtin () {
    clear_screen();
}