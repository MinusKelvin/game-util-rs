use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::channel;

use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use rodio::source::Buffered;
use rodio::{Decoder, OutputStream, Sink, Source};

use crate::sound::SoundCommand;

pub(crate) type InternalSound = Buffered<Decoder<BufReader<File>>>;

pub(crate) async fn load(source: &str) -> Result<InternalSound, String> {
    Ok(Decoder::new(BufReader::new(
        File::open(source).map_err(|e| e.to_string())?,
    ))
    .map_err(|e| e.to_string())?
    .buffered())
}

pub(crate) async fn sound_service(mut source: UnboundedReceiver<SoundCommand>) {
    let (send, recv) = channel();

    std::thread::spawn(move || {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let mut sinks: Vec<Sink> = vec![];
        while let Ok(cmd) = recv.recv() {
            sinks.retain(|s| !s.empty());
            match cmd {
                SoundCommand::Play(sound) => {
                    if let Ok(sink) = Sink::try_new(&handle) {
                        sink.append(sound);
                        sinks.push(sink);
                    }
                }
                SoundCommand::Pause => sinks.iter().for_each(Sink::pause),
                SoundCommand::Resume => sinks.iter().for_each(Sink::play),
                SoundCommand::Stop => sinks.clear(),
                SoundCommand::SetVolume(volume) => sinks.iter().for_each(|s| s.set_volume(volume)),
            }
        }
    });

    while let Some(cmd) = source.next().await {
        send.send(cmd).ok();
    }
}
