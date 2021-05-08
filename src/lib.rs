#![cfg_attr(
    feature = "nightly",
    feature(external_doc),
    deny(missing_docs),
    doc(include = "../README.md")
)]

/// A trait used to simulate monochrome images.
#[auto_impl::auto_impl(&, &mut, Box, Rc, Arc)]
pub trait Image {
    /// Width of the image.
    fn width(&self) -> u32;

    /// Height of the image.
    fn height(&self) -> u32;

    /// Is this pixel considered set or empty.
    fn has_pixel(&self, x: u32, y: u32) -> bool;
}

#[derive(PartialEq, Eq)]
enum Marker {
    Start,
    StartOfSegment,
    EndOfSegment,
    End,
}

#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Pixel coordinates returned to the caller.
pub struct Pixel {
    pub x: u32,
    pub y: u32,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PS {
    Complete,
    Object,
    Incomplete,
}

#[derive(PartialEq, Eq)]
enum CS {
    NonObject,
    Object,
}

struct Range {
    start: u32,
    end: u32,
}

impl From<u32> for Range {
    fn from(start: u32) -> Self {
        Self { start, end: start }
    }
}

struct LutzObject {
    range: Option<Range>,
    info: Vec<Pixel>,
}

struct LutzState<Img> {
    img: Img,
    co: genawaiter::rc::Co<Vec<Pixel>>,
    marker: Box<[Option<Marker>]>,
    obj_stack: Vec<LutzObject>,
    ps: PS,
    cs: CS,
    ps_stack: Vec<PS>,
    store: Box<[Vec<Pixel>]>,
}

impl<Img: Image> LutzState<Img> {
    fn new(img: Img, co: genawaiter::rc::Co<Vec<Pixel>>) -> Self {
        Self {
            co,
            marker: std::iter::repeat_with(|| None)
                .take(img.width() as usize + 1)
                .collect(),
            obj_stack: Vec::new(),
            ps: PS::Complete,
            cs: CS::NonObject,
            ps_stack: Vec::new(),
            store: std::iter::repeat_with(Vec::new)
                .take(img.width() as usize + 1)
                .collect(),
            img,
        }
    }

    async fn run(mut self) {
        let width = self.img.width();
        for y in 0..self.img.height() {
            self.ps = PS::Complete;
            self.cs = CS::NonObject;
            for x in 0..width {
                let newmarker = self.marker[x as usize].take();
                if self.img.has_pixel(x, y) {
                    // Current pixel is part of an object.
                    if self.cs != CS::Object {
                        // Previous pixel is not part of an object, start a new segment.
                        self.start_segment(x);
                    }
                    if let Some(marker) = newmarker {
                        self.process_new_marker(marker, x).await;
                    }
                    // Update current object by current pixel.
                    self.obj_stack.last_mut().unwrap().info.push(Pixel { x, y });
                } else {
                    // Current pixel is not part of an object.
                    if let Some(marker) = newmarker {
                        self.process_new_marker(marker, x).await;
                    }
                    if self.cs == CS::Object {
                        // Previous pixel was part of an object, finish segment.
                        self.end_segment(x);
                    }
                }
            }
            // Handle the extra "M+1" cell from the algorithm
            // (same logic as in the loop above, but without first branch).
            if let Some(marker) = self.marker[width as usize].take() {
                self.process_new_marker(marker, width).await;
            }
            if self.cs == CS::Object {
                self.end_segment(width);
            }
        }
    }

    fn start_segment(&mut self, x: u32) {
        self.cs = CS::Object;
        self.marker[x as usize] = Some(if self.ps == PS::Object {
            // Pixel touches segment on the preceding scan.
            let range = &mut self.obj_stack.last_mut().unwrap().range;
            if range.is_none() {
                // First pixel of object on the current scan.
                *range = Some(Range::from(x));
                Marker::Start
            } else {
                Marker::StartOfSegment
            }
        } else {
            // Start of a completely new object.
            self.ps_stack.push(self.ps);
            self.ps = PS::Complete;
            self.obj_stack.push(LutzObject {
                range: Some(Range::from(x)),
                info: Vec::new(),
            });
            Marker::Start
        });
    }

    fn end_segment(&mut self, x: u32) {
        self.cs = CS::NonObject;
        self.marker[x as usize] = Some(if self.ps != PS::Complete {
            // End of a segment but not necessarily of a section.
            self.obj_stack
                .last_mut()
                .unwrap()
                .range
                .as_mut()
                .unwrap()
                .end = x;
            Marker::EndOfSegment
        } else {
            // End of the final segment of an object section.
            self.ps = self.ps_stack.pop().unwrap();
            let obj = self.obj_stack.pop().unwrap();
            self.store[obj.range.unwrap().start as usize] = obj.info;
            Marker::End
        });
    }

    async fn process_new_marker(&mut self, newmarker: Marker, x: u32) {
        self.ps = match newmarker {
            Marker::Start => {
                // Start of an object on the preceding scan.
                self.ps_stack.push(self.ps);
                let store = std::mem::take(&mut self.store[x as usize]);
                if self.cs == CS::NonObject {
                    // First encounter with this object.
                    self.ps_stack.push(PS::Complete);
                    // Make the object the current object.
                    self.obj_stack.push(LutzObject {
                        range: None,
                        info: store,
                    });
                } else {
                    // Append object to the current object.
                    self.obj_stack.last_mut().unwrap().info.extend(store);
                }
                PS::Object
            }
            Marker::StartOfSegment => {
                // Start of a secondary segment of an object on the preceding scan.
                if self.cs == CS::Object && self.ps == PS::Complete {
                    // Current object is joined to the preceding object.
                    self.ps_stack.pop();
                    let obj = self.obj_stack.pop().unwrap();
                    // Join the two objects.
                    let new_top = self.obj_stack.last_mut().unwrap();
                    new_top.info.extend(obj.info);
                    let k = obj.range.unwrap().start;
                    if new_top.range.is_none() {
                        new_top.range = Some(Range::from(k));
                    } else {
                        self.marker[k as usize] = Some(Marker::StartOfSegment);
                    }
                }
                PS::Object
            }
            Marker::EndOfSegment => PS::Incomplete,
            // Note: there is a typo in the paper, this needs to be 'F' (end) not 'F[0]' again (end of segment).
            Marker::End => {
                // End of an object on the preceding scan.
                let ps = self.ps_stack.pop().unwrap();
                if self.cs == CS::NonObject && ps == PS::Complete {
                    // If there's no more of the current object to come, finish it.
                    let obj = self.obj_stack.pop().unwrap();
                    match obj.range {
                        None => {
                            // Object completed.
                            self.co.yield_(obj.info).await;
                        }
                        Some(range) => {
                            // Object completed on this scan.
                            self.marker[range.end as usize] = Some(Marker::End);
                            self.store[range.start as usize] = obj.info;
                        }
                    }
                    self.ps_stack.pop().unwrap()
                } else {
                    ps
                }
            }
        }
    }
}

/// Main function that performs object detection in the provided image.
pub fn lutz(img: impl Image) -> impl IntoIterator<Item = Vec<Pixel>> {
    genawaiter::rc::Gen::new(move |co| LutzState::new(img, co).run())
}
