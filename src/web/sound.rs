use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::StreamExt;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    AudioBuffer, AudioBufferSourceNode, AudioContext, AudioContextState, ConstantSourceNode,
};
use webutil::event::EventTargetExt;

use crate::sound::SoundCommand;

pub(crate) type InternalSound = AudioBuffer;

thread_local! {
    static AUDIO_CONTEXT: AudioContext = AudioContext::new().unwrap();
}

pub(crate) async fn load(source: &str) -> Result<InternalSound, String> {
    let buffer = super::load_buffer(source).await?;
    JsFuture::from(AUDIO_CONTEXT.with(|ctx| ctx.decode_audio_data(&buffer).unwrap()))
        .await
        .map_err(super::js_err)
        .map(|v| v.dyn_into().unwrap())
}

enum ServiceState {
    Playing(Vec<(AudioBufferSourceNode, f64)>),
    Paused(Vec<(AudioBuffer, f64)>),
}

struct SoundServiceImpl {
    state: ServiceState,
    gain_source: ConstantSourceNode,
    ended_sounds_send: UnboundedSender<AudioBufferSourceNode>,
}

impl SoundServiceImpl {
    fn new(ended_sounds_send: UnboundedSender<AudioBufferSourceNode>) -> Self {
        let gain_source = AUDIO_CONTEXT.with(|ctx| ctx.create_constant_source().unwrap());
        gain_source.start().unwrap();
        Self {
            state: ServiceState::Playing(vec![]),
            gain_source,
            ended_sounds_send,
        }
    }

    fn play(&mut self, sound: AudioBuffer) {
        match &mut self.state {
            ServiceState::Playing(sounds) => {
                let ended_sounds_send = &self.ended_sounds_send;
                let gain_source = &self.gain_source;
                AUDIO_CONTEXT.with(|ctx| {
                    if ctx.state() == AudioContextState::Running {
                        play_sound(ctx, sounds, ended_sounds_send, gain_source, &sound, 0.0)
                    }
                });
            }
            ServiceState::Paused(sounds) => sounds.push((sound, 0.0)),
        }
    }

    fn resume(&mut self) {
        if let ServiceState::Paused(sounds) = &mut self.state {
            let mut playing = vec![];
            let ended_sounds_send = &self.ended_sounds_send;
            let gain_source = &self.gain_source;
            AUDIO_CONTEXT.with(|ctx| {
                for &mut (ref sound, offset) in sounds {
                    play_sound(
                        ctx,
                        &mut playing,
                        ended_sounds_send,
                        gain_source,
                        &sound,
                        offset,
                    );
                }
            });
            self.state = ServiceState::Playing(playing);
        }
    }

    fn pause(&mut self) {
        if let ServiceState::Playing(sounds) = &mut self.state {
            self.state = AUDIO_CONTEXT.with(|ctx| {
                ServiceState::Paused(
                    sounds
                        .iter()
                        .map(|(source, start_time)| {
                            source.stop().unwrap();
                            (source.buffer().unwrap(), ctx.current_time() - start_time)
                        })
                        .collect(),
                )
            });
        }
    }

    fn stop(&mut self) {
        if let ServiceState::Playing(sounds) = &mut self.state {
            sounds.iter().for_each(|(sound, _)| sound.stop().unwrap())
        }
        // this would get cleared anyways since each sound receives the ended event,
        // but it's more efficient and clear to have the list be emptied all at once.
        self.state = ServiceState::Playing(vec![]);
    }

    fn set_volume(&mut self, volume: f32) {
        self.gain_source.offset().set_value(volume);
    }

    fn sound_ended(&mut self, sound: AudioBufferSourceNode) {
        if let ServiceState::Playing(sounds) = &mut self.state {
            sounds.retain(|(s, _)| s != &sound);
        }
    }
}

fn play_sound(
    ctx: &AudioContext,
    playing: &mut Vec<(AudioBufferSourceNode, f64)>,
    ended_sounds_send: &UnboundedSender<AudioBufferSourceNode>,
    gain_source: &ConstantSourceNode,
    sound: &AudioBuffer,
    offset: f64,
) {
    let gain = ctx.create_gain().unwrap();
    gain.connect_with_audio_node(&ctx.destination()).unwrap();
    gain_source.connect_with_audio_param(&gain.gain()).unwrap();

    let source: AudioBufferSourceNode = ctx.create_buffer_source().unwrap().dyn_into().unwrap();
    source.connect_with_audio_node(&gain).unwrap();
    source.set_buffer(Some(sound));

    source
        .start_with_when_and_grain_offset(0.0, offset)
        .unwrap();
    let event = source.once::<webutil::event::Ended>();
    playing.push((source.clone(), ctx.current_time() - offset));
    let ended_sounds_send = ended_sounds_send.clone();
    spawn_local(async move {
        event.await;
        // It's possible the responsible deleter has
        // been dropped, as stopping audio still fires
        // the ended event, so failure is ignored.
        ended_sounds_send.unbounded_send(source).ok();
    });
}

pub(crate) async fn sound_service(mut source: UnboundedReceiver<SoundCommand>) {
    let (ended_sounds_send, mut ended_sounds) = unbounded();
    let mut service = SoundServiceImpl::new(ended_sounds_send);
    loop {
        futures::select! {
            cmd = source.next() => match cmd {
                Some(SoundCommand::Play(sound)) => service.play(sound),
                Some(SoundCommand::Pause) => service.pause(),
                Some(SoundCommand::Resume) => service.resume(),
                Some(SoundCommand::Stop) => service.stop(),
                Some(SoundCommand::SetVolume(volume)) => service.set_volume(volume),
                None => break
            },
            ended = ended_sounds.next() => if let Some(ended) = ended {
                service.sound_ended(ended);
            }
        }
    }
}
