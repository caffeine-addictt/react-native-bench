use anyhow::Result;
use crossterm::cursor;
use std::io::stderr;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};

use crate::cliutil;

pub struct Step {
    name: String,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl Step {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    pub fn show(&mut self) {
        let name = Arc::new(self.name.clone());
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        self.handle = Some(thread::spawn(move || -> Result<()> {
            let mut tick = 0;
            let mut stderr = stderr();

            crossterm::execute!(stderr, cursor::Hide)?;
            while running.load(Ordering::SeqCst) {
                cliutil::write_progress(&mut stderr, &name, tick)?;
                tick = (tick + 1) % cliutil::SPINNER_CHARS.len();
                thread::sleep(cliutil::UPDATE_DELAY);
            }

            crossterm::queue!(stderr, cursor::Show)?;
            cliutil::write_completed(&mut stderr, &name)?;
            Ok(())
        }));
    }

    pub fn stop(&mut self) -> Result<()> {
        if !self.running.swap(false, Ordering::SeqCst) {
            return Ok(());
        }

        let mut errors = vec![];
        if let Some(handle) = self.handle.take() {
            match handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(e)) => errors.push(format!("{:?}", e)),
                Err(panic) => errors.push(format!("thread panicked: {:?}", panic)),
            }
        }

        if !errors.is_empty() {
            anyhow::bail!("{}", errors.join("\n"));
        }
        Ok(())
    }
}

impl Drop for Step {
    fn drop(&mut self) {
        self.stop().expect("Step{} to stop");
    }
}
