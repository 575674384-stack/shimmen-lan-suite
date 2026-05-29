use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::time::Duration;

pub struct FolderWatcher {
    _watcher: RecommendedWatcher,
}

impl FolderWatcher {
    pub fn new(
        folder_path: String,
        on_change: Box<dyn Fn() + Send>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<Event>>();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                let _ = tx.send(res);
            },
            Config::default(),
        )?;

        watcher.watch(Path::new(&folder_path), RecursiveMode::Recursive)?;

        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                loop {
                    match rx.recv() {
                        Ok(Ok(_event)) => {
                            // 防抖：收到事件后等待 500ms，合并连续事件
                            std::thread::sleep(Duration::from_millis(500));
                            while rx.try_recv().is_ok() {}
                            on_change();
                        }
                        Ok(Err(_)) => {
                            // watcher 内部错误，短暂休眠后继续
                            std::thread::sleep(Duration::from_millis(1000));
                        }
                        Err(_) => {
                            // RecvError: watcher 已被释放，退出循环，避免 100% CPU
                            break;
                        }
                    }
                }
            }));
            if let Err(e) = result {
                eprintln!("[file_sync] watcher thread panicked: {:?}", e);
            }
        });

        Ok(Self { _watcher: watcher })
    }
}
