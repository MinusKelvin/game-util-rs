use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::{channel, Receiver};

use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use rodio::source::{Buffered, UniformSourceIterator};
use rodio::{Decoder, OutputStream, Sink, Source};

use crate::sound::SoundCommand;

pub(crate) type InternalSound = Buffered<UniformSourceIterator<Decoder<BufReader<File>>, i16>>;

const CHANNELS: u16 = 2;
const SAMPLE_RATE: u32 = 44_100;

pub(crate) async fn load(source: &str) -> Result<InternalSound, String> {
    Ok(UniformSourceIterator::new(
        Decoder::new(BufReader::new(
            File::open(source).map_err(|e| e.to_string())?,
        ))
        .map_err(|e| e.to_string())?,
        CHANNELS,
        SAMPLE_RATE,
    )
    .buffered())
}

struct Mixer {
    active: Vec<InternalSound>,
    incoming: Receiver<InternalSound>,
}

impl Iterator for Mixer {
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        self.active.extend(self.incoming.try_iter());

        let mut sample_value = 0i16;
        for i in (0..self.active.len()).rev() {
            match self.active[i].next() {
                Some(v) => sample_value = sample_value.saturating_add(v),
                None => {
                    self.active.swap_remove(i);
                }
            }
        }

        Some(sample_value)
    }
}

impl Source for Mixer {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        CHANNELS
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

pub(crate) async fn sound_service(mut source: UnboundedReceiver<SoundCommand>) {
    let (send, recv) = channel();

    std::thread::spawn(move || {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let (mut sound_send, incoming) = channel();
        let sink = Sink::try_new(&handle).unwrap();
        sink.append(Mixer {
            active: vec![],
            incoming,
        });

        while let Ok(cmd) = recv.recv() {
            match cmd {
                SoundCommand::Play(sound) => {
                    sound_send.send(sound).ok();
                }
                SoundCommand::Pause => sink.pause(),
                SoundCommand::Resume => sink.play(),
                SoundCommand::Stop => {
                    sink.stop();
                    sink.play();
                    let (s, incoming) = channel();
                    sound_send = s;
                    sink.append(Mixer {
                        active: vec![],
                        incoming,
                    });
                }
                SoundCommand::SetVolume(volume) => sink.set_volume(volume),
            }
        }
    });

    while let Some(cmd) = source.next().await {
        send.send(cmd).ok();
    }
}
