use anyhow::Result;
use crossterm::cursor;
use std::io::{self, BufRead};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender},
};
use std::sync::{Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossterm::style::{Print, Stylize};
use crossterm::{
    cursor::{MoveToColumn, MoveUp},
    terminal::{Clear, ClearType},
};

use crate::cliutil;

#[derive(Debug)]
pub(crate) struct MultiStep {
    name: String,
    rows: usize,
    running: Arc<AtomicBool>,
    handle: Option<Vec<JoinHandle<Result<()>>>>,
    tx: Option<Sender<String>>,
    buffer: Arc<Mutex<Vec<String>>>,
    out: Arc<RwLock<Vec<String>>>,
}

#[allow(dead_code)]
impl MultiStep {
    pub(crate) fn new(name: impl Into<String>, rows: usize) -> Self {
        Self {
            name: name.into(),
            rows,
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            tx: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            out: Arc::new(RwLock::new(vec![])),
        }
    }

    pub(crate) fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub(crate) fn output(&self) -> String {
        self.out.read().unwrap().join("\n")
    }

    pub(crate) fn show(&mut self) {
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        self.tx = Some(tx.clone());

        let name = Arc::new(self.name.clone());
        let buffer = self.buffer.clone();
        let buffer_2 = self.buffer.clone();

        let running = self.running.clone();
        let running_2 = self.running.clone();
        running.store(true, Ordering::SeqCst);

        let rows = self.rows;
        let out = self.out.clone();
        self.handle = Some(vec![
            // buf updater
            thread::spawn(move || -> Result<()> {
                while running.load(Ordering::SeqCst) {
                    match rx.recv_timeout(Duration::from_millis(200)) {
                        Ok(line) => {
                            let mut b = buffer.lock().unwrap();
                            b.push(line.clone());
                            if b.len() > rows {
                                b.remove(0);
                            }
                            out.write().unwrap().push(line.clone());
                        }

                        Err(RecvTimeoutError::Timeout) => (),
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                }

                Ok(())
            }),
            // out
            thread::spawn(move || -> Result<()> {
                let mut tick = 0;

                let mut stderr = io::stderr();
                let mut content: Vec<String> = vec![String::default(); rows + 1];

                crossterm::execute!(stderr, cursor::Hide)?;
                while running_2.load(Ordering::SeqCst) {
                    let snap = {
                        let buf = buffer_2.lock().unwrap();
                        buf.clone()
                    };

                    content[0] = cliutil::progress_title(&name, tick);
                    for i in 1..=rows {
                        let mut s = "|".white().bold().to_string();
                        if i <= snap.len() {
                            s = format!("{} {}", s, snap[i - 1].to_string().trim_end().dark_grey());
                        }

                        content[i] = s;
                    }

                    crossterm::execute!(
                        stderr,
                        MoveUp(rows as u16 + 1),
                        MoveToColumn(0),
                        Clear(ClearType::FromCursorDown),
                        Print(content.join("\n") + "\n")
                    )?;

                    tick = (tick + 1) % cliutil::SPINNER_CHARS.len();
                    std::thread::sleep(cliutil::UPDATE_DELAY);
                }

                crossterm::queue!(stderr, cursor::Show)?;
                crossterm::execute!(
                    stderr,
                    MoveUp(rows as u16 + 1),
                    MoveToColumn(0),
                    Clear(ClearType::FromCursorDown),
                    Print(cliutil::completed_title(&name))
                )?;

                Ok(())
            }),
        ]);
    }

    /// Register a reader
    ///
    /// All readers will be read concurrently
    /// and are unified into a single output stream
    /// on a first-send-first-out basis.
    pub(crate) fn register_reader<R: io::Read + Send + 'static>(&self, reader: R) {
        let tx = self.tx.clone();

        let mut x = crossterm::terminal::size().unwrap_or((120, 0)).0;
        x = 120.clamp(20, x);

        thread::spawn(move || {
            let reader = io::BufReader::new(reader);
            reader.lines().map_while(Result::ok).for_each(|line| {
                cliutil::split_line_to_chunks(line.as_str(), x as usize)
                    .iter()
                    .for_each(|chunk| {
                        let _ = tx.as_ref().unwrap().send(chunk.to_string());
                    });
            });
        });
    }

    pub(crate) fn send(&self, line: impl Into<String>) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(line.into());
        }
    }

    pub(crate) fn stop(&mut self) -> Result<()> {
        if !self.running.swap(false, Ordering::SeqCst) {
            return Ok(());
        }

        let mut errors = vec![];
        if let Some(handles) = self.handle.take() {
            for x in handles {
                match x.join() {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => errors.push(format!("{:?}", e)),
                    Err(panic) => errors.push(format!("thread panicked: {:?}", panic)),
                }
            }
        }
        self.buffer.lock().unwrap().clear();

        if !errors.is_empty() {
            anyhow::bail!("{}", errors.join("\n"));
        }
        Ok(())
    }
}

impl Drop for MultiStep {
    fn drop(&mut self) {
        self.stop().expect("MultiStep{} to stop");
    }
}
