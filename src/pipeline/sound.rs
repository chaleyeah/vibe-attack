use anyhow::Result;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct SoundPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl SoundPlayer {
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        Ok(Self {
            _stream: stream,
            stream_handle,
        })
    }

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
