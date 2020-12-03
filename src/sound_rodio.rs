use rodio::{ Sink, Source, Sample, Decoder, default_output_device, source::Buffered };
use std::io::Cursor;

pub struct Sound {
    sound: Buffered<Decoder<Cursor<&'static [u8]>>>
}

impl Sound {
    pub fn new(bytes: &'static [u8]) -> Sound {
        Sound {
            sound: Decoder::new(Cursor::new(bytes)).unwrap().buffered()
        }
    }

    pub fn sound(&self) -> impl Source<Item=impl Sample + Send> + Send + 'static {
        self.sound.clone()
    }

    pub fn play(&self) {
        let sink = Sink::new(&default_output_device().unwrap());
        sink.append(self.sound());
        sink.detach();
    }
}