use futures::channel::mpsc::UnboundedReceiver;

use crate::sound::SoundCommand;

// TODO

pub type InternalSound = ();

pub(crate) async fn load(source: &str) -> Result<InternalSound, String> {
    Ok(())
}

pub(crate) async fn sound_service(mut source: UnboundedReceiver<SoundCommand>) {
    
}
