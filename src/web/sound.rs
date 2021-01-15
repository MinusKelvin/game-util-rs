use wasm_bindgen::{JsValue, JsCast};
use js_sys::{Error, Array, ArrayBuffer, Map};
use web_sys::{AudioContext, AudioBuffer, AudioBufferSourceNode, ConstantSourceNode, Response};
use wasm_bindgen_futures::JsFuture;
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use webutil::event::EventTargetExt;

use crate::sound::SoundCommand;

pub(crate) type InternalSound = AudioBuffer;

thread_local! {
    static AUDIO_CONTEXT: AudioContext = AudioContext::new().unwrap();
}

pub(crate) async fn load(source: &str) -> Result<InternalSound, String> {
    async fn load_(source: &str) -> Result<InternalSound, JsValue> {
        let response: Response = JsFuture::from(web_sys::window().unwrap().fetch_with_str(source))
            .await?
            .dyn_into()
            .unwrap();
        let buffer: ArrayBuffer = JsFuture::from(response.array_buffer()?)
            .await?
            .dyn_into()
            .unwrap();
        let buffer = JsFuture::from(AUDIO_CONTEXT.with(|ctx| ctx.decode_audio_data(&buffer))?)
            .await?
            .dyn_into()
            .unwrap();
        Ok(buffer)
    }
    load_(source).await.map_err(|err: JsValue| err.dyn_into::<Error>().unwrap().to_string().into())
}

enum ServiceState {
    Playing(Map),
    Paused(Vec<(AudioBuffer, f64)>)
}

pub(crate) async fn sound_service(mut source: UnboundedReceiver<SoundCommand>) {
    fn play_sound(ctx: &AudioContext, source_nodes: &Map, gain_source: &ConstantSourceNode, sound: &AudioBuffer, offset: f64) {
        let gain = ctx.create_gain().unwrap();
        gain.connect_with_audio_node(&ctx.destination()).unwrap();
        gain_source.connect_with_audio_param(&gain.gain()).unwrap();

        let source: AudioBufferSourceNode = ctx
            .create_buffer_source()
            .unwrap()
            .dyn_into()
            .unwrap();
        source.connect_with_audio_node(&gain).unwrap();
        source.set_buffer(Some(sound));
        
        {
            let source = source.clone();
            let source_nodes = source_nodes.clone();
            let event = source.once::<webutil::event::Ended>();
            source_nodes.set(&source, &(ctx.current_time() - offset).into());
            wasm_bindgen_futures::spawn_local(async move {
                event.await;
                source_nodes.delete(&source);
            });
        }
        source.start_with_when_and_grain_offset(0.0, offset).unwrap();
    }

    let gain_source = AUDIO_CONTEXT.with(|ctx| ctx.create_constant_source().unwrap());
    gain_source.start().unwrap();

    let mut state = ServiceState::Playing(Map::new());
    while let Some(cmd) = source.next().await {
        AUDIO_CONTEXT.with(|ctx| {
            match state {
                ServiceState::Playing(ref source_nodes) => match cmd {
                    SoundCommand::Play(ref sound) => play_sound(ctx, source_nodes, &gain_source, sound, 0.0),
                    SoundCommand::Pause => {
                        let source_nodes = source_nodes
                            .entries()
                            .into_iter()
                            .map(|entry| {
                                let entry: Array = entry
                                    .unwrap().dyn_into().unwrap();
                                let source: AudioBufferSourceNode =
                                    entry.get(0).dyn_into().unwrap();
                                let start_time = entry.get(1).as_f64().unwrap();
                                
                                source.stop().unwrap();
                                (source.buffer().unwrap(), ctx.current_time() - start_time)
                            })
                            .collect();
                        state = ServiceState::Paused(source_nodes);
                    },
                    SoundCommand::Resume => {},
                    SoundCommand::Stop => for source in source_nodes.keys() {
                        source.unwrap().dyn_into::<AudioBufferSourceNode>().unwrap().stop().unwrap();
                    },
                    SoundCommand::SetVolume(volume) => gain_source.offset().set_value(volume)
                }
                ServiceState::Paused(ref mut queued_sounds) => match cmd {
                    SoundCommand::Play(sound) => queued_sounds.push((sound, 0.0)),
                    SoundCommand::Pause => {},
                    SoundCommand::Resume => {
                        let source_nodes = Map::new();
                        for (sound, offset) in queued_sounds {
                            play_sound(ctx, &source_nodes, &gain_source, sound, *offset);
                        }
                        state = ServiceState::Playing(source_nodes);
                    },
                    SoundCommand::Stop => queued_sounds.clear(),
                    SoundCommand::SetVolume(volume) => gain_source.offset().set_value(volume)
                }
            }
        });
    }
}
