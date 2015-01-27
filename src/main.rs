#![allow(unstable, dead_code)]

extern crate libc;

mod args;
mod ansi;
mod matching;
mod screen;

use std::os;
use std::io;
use std::cmp::min;
use screen::Screen;
use screen::Key::*;

fn main() {
  let args = match args::parse_args() {
    Some(args) => args,
    None => {
      os::set_exit_status(1);
      return;
    },
  };

  if args.help { return; }

  let choices = read_choices();
  let initial_search = args.initial_search.clone();
  if args.use_first {
    let matches = matching::compute_matches(&choices, initial_search.as_slice());
    println!("{}", matches[0]);
    return;
  } else {
    event_loop(choices, initial_search.as_slice());
  }
}

fn event_loop(choices: Vec<String>, initial_search: &str) {
  let mut search = String::from_str(initial_search);
  let mut screen = Screen::open_screen();
  let mut index = 0;

  let start_line = screen.height - screen.visible_choices - 1;
  let mut matches_stale = true;
  let mut matches = matching::compute_matches(&choices, search.as_slice());
  loop {
    if matches_stale {
      matches = matching::compute_matches(&choices, search.as_slice());
      matches_stale = false;
    }

    draw_screen(&mut screen, &matches, search.as_slice(), choices.len(), start_line, index);

    let chars = screen.get_buffered_keys();
    for char in chars.iter() {
      match *char {
        Char(x) => {
          search.push(x);
          index = 0;
          matches_stale = true;
        }
        Backspace => { search.pop(); matches_stale = true; }
        Control('h') => { search.pop(); matches_stale = true; }
        Control('u') => { search.clear(); matches_stale = true; }
        Control('c') => { return; }
        Control('n') => { index = min(index + 1, min(screen.visible_choices as usize - 1, matches.len() - 1)); }
        Control('p') => { index = if index == 0 { 0 } else { index - 1 }; }
        Enter => {
          let end_line = start_line + screen.visible_choices;
          screen.move_cursor(end_line, 0);
          screen.write("\n");
          if matches_stale {
            matches = matching::compute_matches(&choices, search.as_slice());
          }
          println!("{}", matches[index]);
          return;
        }
        _ => panic!("Unexpected input"),
      }
    }
  }
}

fn draw_screen(screen: &mut Screen, matches: &Vec<&String>, search: &str, choices: usize, start_line: u16, index: usize) {
  screen.hide_cursor();
  screen.blank_screen(start_line);
  screen.move_cursor(start_line, 0);
  screen.write(format!("> {} ({} choices)\n", search, choices).as_slice());

  print_matches(screen, matches, index);

  screen.move_cursor(start_line, 2 + search.len() as u16);
  screen.show_cursor();
}

fn print_matches(screen: &mut Screen, matches: &Vec<&String>, index: usize) {
  let mut i = 1;
  for choice in matches.iter() {
    if i == index + 1 {
      screen.write_inverted(choice.as_slice());
    } else {
      screen.write(choice.as_slice());
    }
    if i >= screen.visible_choices as usize {
      return;
    } else {
      screen.write("\n");
    }
    i += 1;
  }
}

fn read_choices() -> Vec<String> {
  let mut stdin = io::stdio::stdin();
  let mut lines = Vec::new();

  while let Ok(mut s) = stdin.read_line() {
    trim(&mut s);
    lines.push(s);
  }

  lines
}

fn trim(s: &mut String) {
  while let Some(x) = s.pop() {
    if x != '\n' {
      s.push(x);
      return;
    }
  }
}
