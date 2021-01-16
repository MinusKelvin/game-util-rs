use futures::channel::mpsc::UnboundedReceiver;
use futures::FutureExt;
use futures::StreamExt;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, ConstantSourceNode};
use webutil::channel::{channel, Receiver, Sender};
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
    Playing(SoundHandler),
    Paused(Vec<(AudioBuffer, f64)>),
}

struct SoundHandler {
    playing: Vec<(AudioBufferSourceNode, f64)>,
    dead_nodes_send: Sender<AudioBufferSourceNode>,
    dead_nodes_recv: Receiver<AudioBufferSourceNode>,
}

impl SoundHandler {
    fn new() -> Self {
        let (dead_nodes_send, dead_nodes_recv) = channel();
        Self {
            playing: Vec::new(),
            dead_nodes_send,
            dead_nodes_recv,
        }
    }

    fn unpause(
        ctx: &AudioContext,
        gain_source: &ConstantSourceNode,
        sounds: &[(AudioBuffer, f64)],
    ) -> Self {
        let mut this = SoundHandler::new();
        for (sound, offset) in sounds {
            this.play_sound(ctx, &gain_source, sound, *offset);
        }
        this
    }

    fn pause(&self, ctx: &AudioContext) -> Vec<(AudioBuffer, f64)> {
        self.playing
            .iter()
            .map(|(source, start_time)| {
                source.stop().unwrap();
                (source.buffer().unwrap(), ctx.current_time() - start_time)
            })
            .collect()
    }

    fn stop(&mut self) {
        for (sound, _) in &mut self.playing {
            sound.stop().unwrap();
        }
    }

    async fn collect_dead_nodes(&mut self) {
        while let Some(sound) = self.dead_nodes_recv.recv().await {
            self.playing.retain(|(s, _)| s != &sound);
        }
    }

    fn play_sound(
        &mut self,
        ctx: &AudioContext,
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
        self.playing
            .push((source.clone(), ctx.current_time() - offset));
        let dead_nodes_send = self.dead_nodes_send.clone();
        wasm_bindgen_futures::spawn_local(async move {
            event.await;
            // It's possible the responsible deleter has
            // been dropped, as stopping audio still fires
            // the ended event, so failure is ignored.
            drop(dead_nodes_send.send(source));
        });
    }
}

pub(crate) async fn sound_service(mut source: UnboundedReceiver<SoundCommand>) {
    let gain_source = AUDIO_CONTEXT.with(|ctx| ctx.create_constant_source().unwrap());
    gain_source.start().unwrap();

    let mut state = ServiceState::Playing(SoundHandler::new());
    loop {
        match state {
            ServiceState::Playing(ref mut sounds) => {
                futures::select! {
                    cmd = source.next() => if let Some(cmd) = cmd {
                        match cmd {
                            SoundCommand::Play(ref sound) => {
                                AUDIO_CONTEXT.with(|ctx| {
                                    sounds.play_sound(ctx, &gain_source, sound, 0.0);
                                });
                            },
                            SoundCommand::Pause => {
                                state = AUDIO_CONTEXT.with(|ctx| {
                                    ServiceState::Paused(sounds.pause(ctx))
                                });
                            },
                            SoundCommand::Resume => {},
                            SoundCommand::Stop => sounds.stop(),
                            SoundCommand::SetVolume(volume) => gain_source.offset().set_value(volume)
                        }
                    } else {
                        break;
                    },
                    _ = sounds.collect_dead_nodes().fuse() => {}
                }
            }
            ServiceState::Paused(ref mut sounds) => {
                if let Some(cmd) = source.next().await {
                    match cmd {
                        SoundCommand::Play(sound) => sounds.push((sound, 0.0)),
                        SoundCommand::Pause => {}
                        SoundCommand::Resume => {
                            state = AUDIO_CONTEXT.with(|ctx| {
                                ServiceState::Playing(SoundHandler::unpause(
                                    ctx,
                                    &gain_source,
                                    sounds,
                                ))
                            });
                        }
                        SoundCommand::Stop => sounds.clear(),
                        SoundCommand::SetVolume(volume) => gain_source.offset().set_value(volume),
                    }
                } else {
                    break;
                }
            }
        }
    }
}
