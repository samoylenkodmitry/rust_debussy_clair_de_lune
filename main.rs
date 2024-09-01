use std::{error::Error, time::Duration, sync::{Arc, Mutex}, f32::consts::PI, collections::HashMap};
use nodi::{midly::{Smf, Format}, timers::Ticker, Player, Sheet, Connection, MidiEvent};
use rodio::{OutputStream, Sink, Source};
use rand::Rng;

struct Envelope {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    state: EnvelopeState,
    value: f32,
    sample_rate: u32,
}

enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl Envelope {
    fn new(sample_rate: u32) -> Self {
        Self {
            attack: 0.002,  // Faster attack
            decay: 0.15,    // Longer decay
            sustain: 0.5,   // Lower sustain level
            release: 0.5,   // Longer release
            state: EnvelopeState::Idle,
            value: 0.0,
            sample_rate,
        }
    }

    fn trigger(&mut self) {
        self.state = EnvelopeState::Attack;
        self.value = 0.0;
    }

    fn release(&mut self) {
        self.state = EnvelopeState::Release;
    }

    fn process(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Idle => 0.0,
            EnvelopeState::Attack => {
                self.value += 1.0 / (self.attack * self.sample_rate as f32);
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.state = EnvelopeState::Decay;
                }
                self.value
            }
            EnvelopeState::Decay => {
                self.value -= (1.0 - self.sustain) / (self.decay * self.sample_rate as f32);
                if self.value <= self.sustain {
                    self.value = self.sustain;
                    self.state = EnvelopeState::Sustain;
                }
                self.value
            }
            EnvelopeState::Sustain => self.sustain,
            EnvelopeState::Release => {
                self.value -= self.value / (self.release * self.sample_rate as f32);
                if self.value <= 0.001 {
                    self.value = 0.0;
                    self.state = EnvelopeState::Idle;
                }
                self.value
            }
        }
    }
}

struct Note {
    frequency: f32,
    velocity: f32,
    envelope: Envelope,
    phase: f32,
}

struct Synthesizer {
    sample_rate: u32,
    notes: HashMap<u8, Note>,
    string_resonance: Vec<f32>,
    last_sample: f32,
}

impl Synthesizer {
    fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            notes: HashMap::new(),
            string_resonance: vec![0.0; 1024],  // Reduced resonance buffer
            last_sample: 0.0,
        }
    }

    fn note_on(&mut self, key: u8, freq: f32, velocity: f32) {
        let mut envelope = Envelope::new(self.sample_rate);
        envelope.trigger();
        self.notes.insert(key, Note {
            frequency: freq,
            velocity,
            envelope,
            phase: 0.0,
        });
        println!("Note On: key = {}, freq = {}, velocity = {}", key, freq, velocity);
    }

    fn note_off(&mut self, key: u8) {
        if let Some(note) = self.notes.get_mut(&key) {
            note.envelope.release();
        }
    }

    fn process_sample(&mut self) -> f32 {
        let mut sample = 0.0;
        let mut finished_notes = Vec::new();

        for (&key, note) in self.notes.iter_mut() {
            let envelope_value = note.envelope.process();
            if envelope_value == 0.0 {
                finished_notes.push(key);
            } else {
                note.phase += note.frequency * 2.0 * PI / self.sample_rate as f32;
                note.phase %= 2.0 * PI;
                
                let mut note_sample = 0.0;
                let brightness = note.velocity * 0.5 + 0.5; // Adjust this formula as needed
                note_sample += note.phase.sin();
                note_sample += (note.phase * 2.0).sin() * 0.3 * brightness;
                note_sample += (note.phase * 3.0).sin() * 0.1 * brightness.powi(2);
                
                // Apply key scaling to envelope
                let key_scale = 1.0 - (key as f32 / 127.0) * 0.3;
                sample += note_sample * envelope_value * note.velocity * key_scale;
            }
        }

        for key in finished_notes {
            self.notes.remove(&key);
        }

        // Greatly reduced string resonance
        let resonance_factor = 0.05;
        self.string_resonance.push(sample * resonance_factor);
        self.string_resonance.remove(0);
        sample += self.string_resonance.iter().sum::<f32>() / self.string_resonance.len() as f32;

        // Remove reverb effect entirely

        // Apply low-pass filter
        let alpha = 0.1;
        sample = alpha * sample + (1.0 - alpha) * self.last_sample;
        self.last_sample = sample;

        sample * 0.3 // Adjust overall volume
    }
}

struct SynthSource(Arc<Mutex<Synthesizer>>);

impl Iterator for SynthSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.lock().unwrap().process_sample())
    }
}

impl Source for SynthSource {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { 1 }
    fn sample_rate(&self) -> u32 { self.0.lock().unwrap().sample_rate }
    fn total_duration(&self) -> Option<Duration> { None }
}

struct AudioConnection {
    synth: Arc<Mutex<Synthesizer>>,
}

impl Connection for AudioConnection {
    fn play(&mut self, event: MidiEvent) -> bool {
        match event.message {
            nodi::midly::MidiMessage::NoteOn { key, vel } => {
                let freq = 440.0 * 2.0_f32.powf((key.as_int() as f32 - 69.0) / 12.0);
                let velocity = vel.as_int() as f32 / 127.0;
                self.synth.lock().unwrap().note_on(key.as_int(), freq, velocity);
            }
            nodi::midly::MidiMessage::NoteOff { key, .. } => {
                self.synth.lock().unwrap().note_off(key.as_int());
            }
            _ => {}
        }
        true
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = "DEB_CLAI.MID";
    let data = std::fs::read(file)?;
    let Smf { header, tracks } = Smf::parse(&data)?;
    let timer = Ticker::try_from(header.timing)?;

    println!("Parsed MIDI file: {} tracks", tracks.len());
    println!("Format: {:?}, timing: {:?}", header.format, header.timing);

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let synth = Arc::new(Mutex::new(Synthesizer::new(44100)));
    sink.append(SynthSource(synth.clone()));

    let connection = AudioConnection { synth };

    let sheet = match header.format {
        Format::SingleTrack | Format::Sequential => Sheet::sequential(&tracks),
        Format::Parallel => Sheet::parallel(&tracks),
    };

    let mut player = Player::new(timer, connection);

    println!("Starting playback of {}", file);
    player.play(&sheet);
    
    // Keep the program running while audio plays
    std::thread::sleep(Duration::from_secs(30));
    
    println!("Playback finished");
    Ok(())
}