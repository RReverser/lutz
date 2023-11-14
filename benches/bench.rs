use iai::main;
use image::GenericImageView;
use std::sync::OnceLock;

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
struct Pixel {
    x: u32,
    y: u32,
}

struct Image(image::GrayImage);

impl lutz::Image for Image {
    fn width(&self) -> u32 {
        self.0.width()
    }

    fn height(&self) -> u32 {
        self.0.height()
    }

    fn has_pixel(&self, x: u32, y: u32) -> bool {
        unsafe { self.0.unsafe_get_pixel(x, y) }.0[0] > 170
    }
}

static IMG: OnceLock<Image> = OnceLock::new();

fn m100() -> Vec<Vec<lutz::Pixel>> {
    let img = IMG.get().unwrap();
    let mut res = Vec::new();
    for obj_pixels in lutz::lutz(img) {
        res.push(obj_pixels);
    }
    res
}

fn main() {
    IMG.set(Image(
        image::open("examples/m100.png").unwrap().into_luma8(),
    ))
    .unwrap_or_else(|_| unreachable!());
    main!(m100);
    main()
}
