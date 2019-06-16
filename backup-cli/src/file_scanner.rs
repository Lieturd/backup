//use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

// use walkdir::{WalkDir, IntoIter as WalkerIter};

struct FileScannerThread {
    paths: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct FileScanner {
    stop_tx: Sender<bool>,
    read_rx: Receiver<PathBuf>,
}

impl FileScanner {
    pub fn new(paths: Vec<PathBuf>) -> FileScanner {
        let (stop_tx, stop_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
        let (read_tx, read_rx): (Sender<PathBuf>, Receiver<PathBuf>) = mpsc::channel();

        thread::spawn(move || {
            let fst = FileScannerThread {
                paths
            };

            fst.run(stop_rx, read_tx)
        });

        FileScanner {
            stop_tx,
            read_rx,
        }
    }

    pub fn get_receiver(&mut self) -> &Receiver<PathBuf> {
        &self.read_rx
    }

    pub fn stop(&mut self) {
        println!("Stopping file scanner...");
        self.stop_tx.send(true).unwrap();
    }
}

impl Drop for FileScanner {
    fn drop(&mut self) {
        self.stop();
    }
}

impl FileScannerThread {
    fn run(self, stop_rx: Receiver<bool>, read_tx: Sender<PathBuf>) {
        let mut stop = false;
        while !stop {
            for path in &self.paths {
                if FileScannerThread::check_stop(&stop_rx) {
                    stop = true;
                    break;
                }

                //println!("Path {:?} changed", path);
                read_tx.send(path.clone()).unwrap();

                // TODO: Recurse
                // TODO: Speed limiter
            }
        }
        println!("File scanner stopped.");
    }

    fn check_stop(stop_rx: &Receiver<bool>) -> bool {
        match stop_rx.try_recv() {
            Ok(_) => {
                return true;
            }
            Err(_) => {
                return false;
            }
        }
    }
}
