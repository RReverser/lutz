use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
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
    type Pixel = Pixel;

    fn width(&self) -> usize {
        self.0.width() as _
    }

    fn height(&self) -> usize {
        self.0.height() as _
    }

    fn pixel(&self, x: usize, y: usize) -> Option<Self::Pixel> {
        if self.0.get_pixel(x as _, y as _).0[0] > 170 {
            Some(Pixel {
                x: x as _,
                y: y as _,
            })
        } else {
            None
        }
    }
}

fn m100(c: &mut Criterion) {
	c.bench_with_input(BenchmarkId::new("lutz", "m100"), &Image(image::open("m100.png").unwrap().into_luma8()), |b, img| {
		b.iter(|| {
			let mut res = Vec::new();
			lutz::lutz(img, |pixels| {
				res.push(pixels);
			});
			res
		});
	});
	println!("Peak usage: {} MB", PEAK_ALLOC.peak_usage_as_mb());
}

criterion_group!(benches, m100);
criterion_main!(benches);
