use scap::capturer::{Area, Capturer, Options, Point, Size};
use scap::frame::Frame;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::u8;

struct StreamingService {
    capturer: Capturer,
    ffmpeg: Child,
}

impl StreamingService {
    fn new() -> Self {
        let options = Options {
            fps: 60,
            target: None,
            show_cursor: true,
            show_highlight: true,
            excluded_targets: None,
            output_type: scap::frame::FrameType::BGRAFrame,
            output_resolution: scap::capturer::Resolution::_720p,
            crop_area: Some(Area {
                origin: Point { x: 0.0, y: 0.0 },
                size: Size {
                    width: 1920.0,
                    height: 1080.0,
                },
            }),
            ..Default::default()
        };

        let capturer = Capturer::build(options).expect("Failed to create capturer");
        let ffmpeg = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "bgra",
                "-s",
                "1920x1080",
                "-r",
                "60",
                "-i",
                "pipe:0",
                "-c:v",
                "libvpx-vp9",
            ])
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to start FFmpeg");

        Self { capturer, ffmpeg }
    }
    fn start_stream(&mut self) {
        // Check if the platform is supported
        if !scap::is_supported() {
            println!("❌ Platform not supported");
            return;
        }

        // Request capture permissions if necessary
        if !scap::has_permission() && !scap::request_permission() {
            println!("❌ Permission denied");
            return;
        }

        // Start capture
        self.capturer.start_capture();
        println!("🎥 Streaming started. Press Enter to stop...");
        loop {
            match self.capturer.get_next_frame() {
                Ok(frame) => match frame {
                    Frame::BGRA(frame) => {
                        if let Err(e) = self.encoding_ffmpeg(&frame.data) {
                            eprintln!("Error encoding BGRA frame: {}", e);
                            break;
                        }
                    }
                    Frame::YUVFrame(frame) => {
                        if let Err(e) = self.encoding_ffmpeg(&frame.luminance_bytes) {
                            eprintln!("Error encoding YUV frame: {}", e);
                            break;
                        }
                    }
                    Frame::RGB(frame) => {
                        if let Err(e) = self.encoding_ffmpeg(&frame.data) {
                            eprintln!("Error encoding RGB frame: {}", e);
                            break;
                        }
                    }
                    Frame::XBGR(frame) => {
                        if let Err(e) = self.encoding_ffmpeg(&frame.data) {
                            eprintln!("Error encoding XBGR frame: {}", e);
                            break;
                        }
                    }
                    Frame::BGRx(frame) => {
                        if let Err(e) = self.encoding_ffmpeg(&frame.data) {
                            eprintln!("Error encoding BGRx frame: {}", e);
                            break;
                        }
                    }
                    Frame::BGR0(frame) => {
                        if let Err(e) = self.encoding_ffmpeg(&frame.data) {
                            eprintln!("Error encoding BGR0 frame: {}", e);
                            break;
                        }
                    }
                    _ => {
                        eprintln!("Not supported frame format.");
                    }
                },
                Err(e) => {
                    eprintln!("Failed to capture frame: {}", e);
                    break;
                }
            }
        }
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        // Stop the capture
        self.capturer.stop_capture();
        println!("🛑 Streaming stopped.");
    }

    fn encoding_ffmpeg(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ffmpeg_stdin) = self.ffmpeg.stdin.as_mut() {
            // Write frame data to FFmpeg's stdin
            ffmpeg_stdin.write_all(data).map_err(|e| {
                eprintln!("Failed to write frame data to FFmpeg stdin: {}", e);
                e
            })?;
            Ok(())
        } else {
            eprintln!("FFmpeg stdin is not available");
            Err("FFmpeg stdin unavailable".into())
        }
    }
    // fn rtc_service<T>(&self, data: T) {}

    // fn reflect_inputs(&self) {
    //     // Implement input reflection logic here
    //     println!("Reflecting inputs stub.");
    // }
}
fn main() {
    let mut streamer = StreamingService::new();
    streamer.start_stream();
}
