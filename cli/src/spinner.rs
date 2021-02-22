use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use console::Term;

enum SpinnerMsg {}

type SpinnerSender = mpsc::Sender<SpinnerMsg>;
type SpinnerReceiver = mpsc::Receiver<SpinnerMsg>;
type SpinnerChannel = (SpinnerSender, SpinnerReceiver);

#[derive(Clone)]
pub struct SpinnerBuilder {
    spinner_chars: Vec<String>,
    interval: Duration,
    prefix: String,
    text: String,
}

impl SpinnerBuilder {
    pub fn new() -> Self {
        Self {
            interval: Duration::from_millis(100),
            prefix: Default::default(),
            text: Default::default(),
            spinner_chars: "-\\|/".chars()
                .map(|x| x.to_string())
                .collect()
        }
    }

    pub fn with_prefix<T>(mut self, prefix: T) -> Self
        where T: ToString,
    {
        self.prefix = prefix.to_string();
        self
    }

    pub fn with_text<T>(mut self, text: T) -> Self
        where T: ToString,
    {
        self.text = text.to_string();
        self
    }

    pub fn with_interval_ms(mut self, millis: u64) -> Self {
        self.interval = Duration::from_millis(millis);
        self
    }

    pub fn with_spinner_chars<T>(mut self, chars: Vec<T>) -> Self
        where T: ToString,
    {
        self.spinner_chars = chars.into_iter()
            .map(|x| x.to_string())
            .collect();
        self
    }

    pub fn start(self) -> Spinner {
        let (tx, rx): SpinnerChannel = mpsc::channel();
        let spinner_chars = self.spinner_chars;
        let interval = self.interval;
        let prefix = self.prefix;
        let text = self.text;

        let th = thread::spawn(move || {
            let mut has_printed = false;
            for sp_char in spinner_chars.iter().cycle() {
                loop {
                    match rx.try_recv() {
                        Err(mpsc::TryRecvError::Disconnected) => {
                            if has_printed {
                                let _ = Term::stderr().clear_last_lines(1);
                            }
                            return;
                        }
                        _ => break,
                    }
                }

                let t = Term::stderr();

                if has_printed {
                    let _ = t.clear_last_lines(1);
                }
                let _ = t.write_line(&format!(
                    "{} {}   {}",
                    prefix,
                    sp_char,
                    text,
                ));
                has_printed = true;
                thread::sleep(interval);
            }
        });

        Spinner {
            tx: Some(tx),
            thread_handle: Some(th),
        }
    }
}

pub struct Spinner {
    tx: Option<SpinnerSender>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    pub fn finish(self) {
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            drop(tx);
            if let Some(handle) = self.thread_handle.take() {
                let _ = handle.join();
            }
        }
    }
}
