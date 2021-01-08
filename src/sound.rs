use futures::channel::mpsc::{unbounded, UnboundedSender};

use crate::backend::sound as backend;
use crate::LocalExecutor;

pub struct SoundService {
    send: UnboundedSender<SoundCommand>,
}

pub struct Sound {
    sound: backend::InternalSound,
}

impl SoundService {
    pub fn new(executor: &LocalExecutor) -> Self {
        let (send, recv) = unbounded();
        executor.spawn(backend::sound_service(recv));
        SoundService { send }
    }

    pub fn play(&self, sound: &Sound) {
        self.send
            .unbounded_send(SoundCommand::Play(sound.sound.clone()))
            .ok();
    }

    pub fn pause(&self) {
        self.send.unbounded_send(SoundCommand::Pause).ok();
    }

    pub fn resume(&self) {
        self.send.unbounded_send(SoundCommand::Resume).ok();
    }

    pub fn stop(&self) {
        self.send.unbounded_send(SoundCommand::Stop).ok();
    }

    pub fn set_volume(&self, volume: f32) {
        self.send
            .unbounded_send(SoundCommand::SetVolume(volume))
            .ok();
    }
}

impl Sound {
    pub async fn load(source: &str) -> Result<Self, String> {
        backend::load(source).await.map(|sound| Sound { sound })
    }
}

pub(crate) enum SoundCommand {
    Play(backend::InternalSound),
    Pause,
    Resume,
    Stop,
    SetVolume(f32),
}
