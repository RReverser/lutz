use iai::main;
use image::GenericImageView;
use once_cell::sync::OnceCell;
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
        unsafe { self.0.unsafe_get_pixel(x as _, y as _) }.0[0] > 170
    }
}

static IMG: OnceCell<Image> = OnceCell::new();

fn m100() -> Vec<Vec<lutz::Pixel>> {
    let img = IMG.get().unwrap();
    let mut res = Vec::new();
    lutz::lutz(img, |pixels| {
        res.push(pixels);
    });
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
