use anyhow::Result;
use std::{io::Write, time::Duration};

use crossterm::{
    cursor::MoveToColumn,
    style::{Print, Stylize},
    terminal::{Clear, ClearType},
};

pub(crate) const SPINNER_CHARS: &[char; 10] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
pub(crate) const UPDATE_DELAY: Duration = Duration::from_millis(100);

pub(crate) fn progress_title(title: &str, tick: usize) -> String {
    format!(
        "{} {}{}",
        SPINNER_CHARS[tick % SPINNER_CHARS.len()],
        title,
        ".".repeat((tick % 3) + 1)
    )
}

pub(crate) fn completed_title(title: &str) -> String {
    format!("{} {}\n", "[OK]".green().bold(), title)
}

pub(crate) fn write_progress<W: Write>(w: &mut W, title: &str, tick: usize) -> Result<()> {
    Ok(crossterm::execute!(
        w,
        MoveToColumn(0),
        Clear(ClearType::CurrentLine),
        Print(progress_title(title, tick))
    )?)
}

pub(crate) fn write_completed<W: Write>(w: &mut W, title: &str) -> Result<()> {
    Ok(crossterm::execute!(
        w,
        MoveToColumn(0),
        Clear(ClearType::CurrentLine),
        Print(completed_title(title))
    )?)
}

pub(crate) fn split_line_to_chunks<'a, S: Into<&'a str>>(l: S, max_width: usize) -> Vec<String> {
    let graphemes: Vec<&str> =
        unicode_segmentation::UnicodeSegmentation::graphemes(l.into(), true).collect();
    graphemes.chunks(max_width).map(|c| c.concat()).collect()
}
