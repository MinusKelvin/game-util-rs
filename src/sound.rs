use rodio::{ Sink, Source, Sample, Decoder, default_output_device };
use std::io::Cursor;

pub struct Sound {
    bytes: &'static [u8]
}

impl Sound {
    pub fn new(bytes: &'static [u8]) -> Sound {
        Sound {
            bytes
        }
    }

    pub fn sound(&self) -> impl Source<Item=impl Sample + Send> + Send + 'static {
        Decoder::new(Cursor::new(self.bytes)).unwrap()
    }

    pub fn play(&self) {
        let sink = Sink::new(&default_output_device().unwrap());
        sink.append(self.sound());
        sink.detach();
    }
}