use image_compare::Algorithm;
use serde::Serialize;
use sqlx::FromRow;
use xcap::{image::DynamicImage, Monitor};

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct LiveUser {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub hook: String,
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
        if result.score < 0.95 {
            return true;
        }
    }

    false
}
