use iai::main;
use peak_alloc::PeakAlloc;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
struct Pixel {
    x: u32,
    y: u32,
}

struct Image(image::GrayImage);

impl lutz::Image for Image {
    fn width(&self) -> usize {
        self.0.width() as _
    }

    fn height(&self) -> usize {
        self.0.height() as _
    }

    fn has_pixel(&self, x: usize, y: usize) -> bool {
        self.0.get_pixel(x as _, y as _).0[0] > 170
    }
}

fn m100() -> Vec<Vec<lutz::Pixel>> {
    let img = &Image(image::open("m100.png").unwrap().into_luma8());
    let mut res = Vec::new();
    lutz::lutz(img, |pixels| {
        res.push(pixels);
    });
    res
}

main!(m100);
