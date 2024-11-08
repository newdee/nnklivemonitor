use chrono::NaiveDateTime;
use image_compare::Algorithm;
use reqwest::Client;
use serde::Serialize;
use sqlx::FromRow;
use xcap::{image::DynamicImage, Monitor};

#[derive(Serialize)]
pub struct Message {
    pub name: String,
    pub url: String,
    pub updated_at: NaiveDateTime,
    pub desp: String,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct LiveUser {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub hook: String,
    pub status: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub fn area_shot() -> DynamicImage {
    let image = Monitor::from_point(0, 0).unwrap().capture_image().unwrap();
    DynamicImage::from(image).crop(171, 387, 377, 353)
}

pub fn compare_images() -> bool {
    let target = area_shot().into_luma8();
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_secs(2));
        let current = area_shot().into_luma8();
        let result =
            image_compare::gray_similarity_structure(&Algorithm::MSSIMSimple, &target, &current)
                .expect("Images had different dimensions");
        println!("current score: {}", result.score);
        if result.score < 0.95 {
            return true;
        }
    }

    false
}

pub async fn hook_msg(msg: Message, url: String) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let _res = client.post(&url).json(&msg).send().await?;
    Ok(())
}
