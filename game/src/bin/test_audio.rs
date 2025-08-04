//! Simple audio test to verify rodio is working

#[cfg(not(feature = "audio"))]
fn main() {
    println!("This test requires the audio feature to be enabled.");
    println!("Run with: cargo run --bin test_audio --features audio");
}

#[cfg(feature = "audio")]
use rodio::{Decoder, OutputStream, Source};
#[cfg(feature = "audio")]
use std::fs::File;
#[cfg(feature = "audio")]
use std::io::BufReader;
#[cfg(feature = "audio")]
use std::time::Duration;

#[cfg(feature = "audio")]
fn main() {
    println!("=== Audio System Test ===");
    println!("Testing basic audio playback with rodio...\n");

    // Get a output stream handle to the default physical sound device
    match OutputStream::try_default() {
        Ok((_stream, stream_handle)) => {
            println!("✓ Successfully connected to audio device");

            // Try to load the MP3 file
            println!("Loading audio file: game/assets/sounds/ambient_hum.mp3");
            match File::open("game/assets/sounds/ambient_hum.mp3") {
                Ok(file) => {
                    let file = BufReader::new(file);

                    // Decode that sound file into a source
                    match Decoder::new(file) {
                        Ok(source) => {
                            println!("✓ Successfully decoded MP3 file");

                            // Play the sound directly on the device
                            match stream_handle.play_raw(source.convert_samples()) {
                                Ok(_) => {
                                    println!("✓ Audio playback started");
                                    println!(
                                        "\n♪ Playing for 5 seconds... (you should hear sound now)"
                                    );
                                    std::thread::sleep(Duration::from_secs(5));
                                    println!("\n✓ Test completed successfully!");
                                }
                                Err(e) => println!("✗ Failed to play audio: {}", e),
                            }
                        }
                        Err(e) => println!("✗ Failed to decode MP3: {}", e),
                    }
                }
                Err(e) => {
                    println!("✗ Failed to open audio file: {}", e);
                    println!("\nTrying WAV file instead...");

                    // Try WAV file as fallback
                    match File::open("game/assets/sounds/ambient_hum.wav") {
                        Ok(file) => {
                            let file = BufReader::new(file);
                            match Decoder::new(file) {
                                Ok(source) => {
                                    println!("✓ Successfully decoded WAV file");
                                    match stream_handle.play_raw(source.convert_samples()) {
                                        Ok(_) => {
                                            println!("\n♪ Playing WAV for 5 seconds...");
                                            std::thread::sleep(Duration::from_secs(5));
                                            println!("\n✓ WAV playback successful!");
                                        }
                                        Err(e) => println!("✗ Failed to play WAV: {}", e),
                                    }
                                }
                                Err(e) => println!("✗ Failed to decode WAV: {}", e),
                            }
                        }
                        Err(e) => println!("✗ Failed to open WAV file: {}", e),
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to connect to audio device: {}", e);
            println!("\nPossible causes:");
            println!("- No audio output device available");
            println!("- Audio device is in use by another application");
            println!("- Audio drivers not properly installed");
        }
    }

    println!("\nPress Enter to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
}
