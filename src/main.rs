// use ffmpeg_next as ffmpeg;
use scrap::{Capturer, Display};
use std::{
    io::{ErrorKind::WouldBlock, Write},
    process::{self, Command, Stdio},
};
use tokio::time::{sleep, Duration};


#[tokio::main]
async fn main() {
    let display = Display::primary().unwrap_or_else(|error| {
        println!("❌ Could not detect your primary screen.\n Details: {error}");
        process::exit(1);
    });

    let mut capturer = Capturer::new(display).unwrap();
    let width = capturer.width();
    let height = capturer.height();

    println!("Capture, {width}, {height}");

    let child = Command::new("ffplay")
        .args(&[
            "-f",
            "rawvideo",
            "-pixel_format",
            "bgr0",
            "-video_size",
            &format!("{}x{}", width, height),
            "-framerate",
            "24",
            "-",
        ])
        .stdin(Stdio::piped())
        .spawn()
        .expect("❌ Could not play ffplay");
    let mut write = child.stdin.unwrap();

    loop {
        match capturer.frame() {
            Ok(frame) => {
                let stride = frame.len() / height; //number of byte per row in the frame with padding
                let rowlen = 4 * width; //row length, each row takes 4 bytes in bgr0, without padding

                for row in frame.chunks(stride) {
                    let row = &row[..rowlen];
                    write.write_all(row).unwrap();
                }
            }
            Err(ref e) if e.kind() == WouldBlock => {
                //wait for the frame
                println!("🔎 Waiting for frame to arrive!");
                sleep(Duration::from_millis(5)).await;
            }
            Err(e) => {
                eprint!("❌ Something went wrong, Could not capture the frame.\n Details: {e}");
                break;
            }
        }
    }
}
