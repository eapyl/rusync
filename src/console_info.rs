//! console_info
//!
//! Display transfer progress to the command line

use crate::progress::{Progress, ProgressInfo};
use crate::sync;
use colored::Colorize;
use std::io;
use std::io::Write;
use term_size;

#[derive(Default)]
pub struct ConsoleProgressInfo {}

impl ConsoleProgressInfo {
    pub fn new() -> ConsoleProgressInfo {
        ConsoleProgressInfo {}
    }
}

impl ProgressInfo for ConsoleProgressInfo {
    fn done_syncing(&self) {
        erase_line();
    }

    fn start(&self, source: &str, destination: &str) {
        println!(
            "{} Syncing from {} to {} …",
            "::".color("blue"),
            source.bold(),
            destination.bold()
        )
    }

    fn new_file(&self, _name: &str) {}

    fn progress(&self, progress: &Progress) {
        let eta_str = human_seconds(progress.eta);
        let percent_width = 3;
        let eta_width = eta_str.len();
        let index = progress.index;
        let index_width = index.to_string().len();
        let num_files = progress.num_files;
        let num_files_width = num_files.to_string().len();
        let widgets_width = percent_width + index_width + num_files_width + eta_width;
        let num_separators = 5;
        let line_width = get_terminal_width();
        let file_width = line_width - widgets_width - num_separators - 1;
        let current_file = progress.current_file.clone();
        let current_file = truncate_lossy(&current_file, file_width as usize);
        let current_file = format!(
            "{filename:<pad$}",
            pad = file_width as usize,
            filename = current_file
        );
        let file_percent = ((progress.file_done * 100) as usize) / progress.file_size;
        print!(
            "{:>3}% {}/{} {} {:<}\r",
            file_percent, index, num_files, current_file, eta_str
        );
        let _ = io::stdout().flush();
    }

    fn end(&self, stats: &sync::Stats) {
        println!(
            "{} Synced {} files ({} up to date)",
            " ✓".color("green"),
            stats.num_synced,
            stats.up_to_date
        );
        println!(
            "{} files copied, {} symlinks created, {} symlinks updated",
            stats.copied, stats.symlink_created, stats.symlink_updated
        );
    }
}

fn get_terminal_width() -> usize {
    if let Some((w, _)) = term_size::dimensions() {
        return w;
    }
    // We're likely not a tty here, so this is a good enough
    // default:
    80
}

fn erase_line() {
    let line_width = get_terminal_width();
    let line = vec![32 as u8; line_width as usize];
    // We're calling from_utf8 on a string containing only spaces,
    // so calling unwrap() is safe
    print!("{}\r", String::from_utf8(line).unwrap());
}

fn human_seconds(s: usize) -> String {
    let hours = s / 3600;
    let minutes = (s / 60) % 60;
    let seconds = s % 60;
    return format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
}

fn truncate_lossy(text: &str, maxsize: usize) -> String {
    // Our goal here is to make sure the text can be written
    // in the terminal without going over the `maxsize` length
    // Our approach is to first convert to bytes, then truncate
    // the vector of bytes, then convert to a lossy string
    // This way we *know* we won't cut at a char boundary
    let mut as_bytes = text.to_string().into_bytes();
    as_bytes.truncate(maxsize);
    String::from_utf8_lossy(&as_bytes).to_string()
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_truncate_string() {
        let new_text = truncate_lossy("ééé", 2);
        assert_eq!(new_text, "é");
    }

    #[test]
    fn test_human_seconds() {
        assert_eq!("00:00:05", human_seconds(5));
        assert_eq!("00:00:42", human_seconds(42));
        assert_eq!("00:03:05", human_seconds(185));
        assert_eq!("02:04:05", human_seconds(7445));
        assert_eq!("200:00:02", human_seconds(720_002));
    }

}
