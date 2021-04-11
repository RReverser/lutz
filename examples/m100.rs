use std::fs::File;
use std::io::Write;

struct Image(image::GrayImage);

impl lutz::Image for Image {
    fn width(&self) -> u32 {
        self.0.width()
    }

    fn height(&self) -> u32 {
        self.0.height()
    }

    fn has_pixel(&self, x: u32, y: u32) -> bool {
        self.0.get_pixel(x, y).0[0] > 170
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut log = File::create("examples/m100.log")?;
    let mut img = image::open("examples/m100.png")?;
    lutz::lutz(&Image(img.to_luma8()), |pixels| {
        writeln!(log, "{} {:?}", pixels.len(), pixels).unwrap();

        let mut min_x = u32::max_value();
        let mut min_y = u32::max_value();
        let mut max_x = 0;
        let mut max_y = 0;

        for pixel in pixels {
            min_x = min_x.min(pixel.x);
            min_y = min_y.min(pixel.y);
            max_x = max_x.max(pixel.x);
            max_y = max_y.max(pixel.y);
        }

        let rect = imageproc::rect::Rect::at(min_x as i32, min_y as i32)
            .of_size((max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32);
        imageproc::drawing::draw_hollow_rect_mut(&mut img, rect, image::Rgba([255, 0, 0, 255]));
    });
    img.save("examples/m100.out.png")?;
    Ok(())
}
