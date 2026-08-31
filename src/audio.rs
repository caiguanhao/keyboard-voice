use crate::keymap::KeySpec;
use std::{
    io::Cursor,
    sync::mpsc::{sync_channel, SyncSender},
    thread,
};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/audio_assets.rs"));
}

pub struct AudioPlayer {
    sender: SyncSender<KeySpec>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (sender, receiver) = sync_channel::<KeySpec>(32);
        thread::Builder::new()
            .name("keyboard-voice-audio".into())
            .spawn(move || {
                let output = rodio::OutputStream::try_default().ok();
                let sink = output
                    .as_ref()
                    .and_then(|(_, handle)| rodio::Sink::try_new(handle).ok());

                if output.is_none() && !cfg!(target_os = "macos") {
                    eprintln!("No audio output device was found");
                }

                while let Ok(spec) = receiver.recv() {
                    let Some(sink) = &sink else { continue };
                    let Some(bytes) = generated::audio_bytes(spec.id) else {
                        continue;
                    };
                    let Ok(source) = rodio::Decoder::new(Cursor::new(bytes)) else {
                        continue;
                    };
                    // Keep the latest key immediate instead of queueing a full
                    // phrase behind earlier key presses.
                    sink.clear();
                    sink.append(source);
                    sink.play();
                }
            })
            .expect("spawn audio thread");
        Self { sender }
    }

    pub fn speak(&self, spec: KeySpec) {
        let _ = self.sender.try_send(spec);
    }
}
