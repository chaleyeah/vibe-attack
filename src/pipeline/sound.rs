use anyhow::Result;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Plays audio files (e.g. macro confirmation sounds) via the default system output device.
///
/// Wraps a `rodio` output stream. The stream must be kept alive as long as audio may play;
/// dropping it silences any in-flight audio. `SoundPlayer` is intentionally `!Send` because
/// `rodio::OutputStream` is `!Send` — see `SAFETY` comment on `Dispatcher`.
pub struct SoundPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl SoundPlayer {
    /// Open the default audio output device and return a ready-to-use player.
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        Ok(Self {
            _stream: stream,
            stream_handle,
        })
    }

    /// Decode and play the audio file at `path` in a fire-and-forget `Sink`.
    ///
    /// Returns immediately; the audio continues playing on the background rodio thread.
    pub fn play<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader)?;

        // Fire and forget via Sink detached
        let sink = Sink::try_new(&self.stream_handle)?;
        sink.append(decoder);
        sink.detach();

        Ok(())
    }
}
