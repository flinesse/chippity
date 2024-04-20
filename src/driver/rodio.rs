use crate::driver::{AudioDevice, AudioInfo};

pub struct Rodio {
    // Output audio source
    _stream: rodio::OutputStream,
    // Handle to audio device which controls playback
    // TODO: rodio only provides sine wave synthesis, but we can approximate
    // square/triangle/sawtooth waves for a more retro feel
    //   see https://observablehq.com/@freedmand/sounds
    sink: rodio::Sink,
}

impl Rodio {
    pub fn new() -> Self {
        use rodio::Source;

        let (stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();

        // F4 pure tone
        let source = rodio::source::SineWave::new(349.23).amplify(0.1);
        sink.append(source);
        sink.pause();

        Rodio {
            _stream: stream,
            sink,
        }
    }
}

impl AudioDevice for Rodio {
    fn receive_signal(&mut self, data: bool) -> &mut dyn AudioDevice {
        match data {
            true => self.sink.play(),
            false => self.sink.pause(),
        }

        self
    }

    fn play_audio(&mut self) {}

    fn device_info(&self) -> AudioInfo {
        AudioInfo::Rodio
    }
}
