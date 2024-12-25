# Lutz

This is a Rust implementation of "An Algorithm for the Real Time Analysis of Digitised Images" by R. K. Lutz.

It's a single-pass algorithm for [connected-component labeling](https://en.wikipedia.org/wiki/Connected-component_labeling) that allows to find 8-connected objects in a binary (monochrome) image.

## Usage

Crate expects the user to implement its `lutz::Image` trait. A possible implementation for a struct wrapping an [`image::GrayImage`](https://docs.rs/image/0.23.14/image/type.GrayImage.html) type:

```rust
struct Image {
    img: image::GrayImage,
    threshold: u8,
}

impl lutz::Image for Image {
    fn width(&self) -> u32 {
        self.img.width()
    }

    fn height(&self) -> u32 {
        self.img.height()
    }

    fn has_pixel(&self, x: u32, y: u32) -> bool {
        self.img.get_pixel(x, y).0[0] > self.threshold
    }
}
```

Once constructed, a reference to such image should be passed to the `lutz` function. It will return an iterator over detected objects, each represented as a `Vec<Pixel>` of its pixel coordinates:

```rust,ignore
for obj_pixels in lutz::lutz::<_, Vec<_>>(&img) {
    println!("{:?}", obj_pixels);
}
```
