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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = image::open("m100.png")?;
    lutz::lutz(&Image(img.to_luma8()), |pixels| {
        println!("{} {:?}", pixels.len(), pixels);

        let mut min_x = usize::max_value();
        let mut min_y = usize::max_value();
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
    img.save("m100.out.png")?;
    Ok(())
}
