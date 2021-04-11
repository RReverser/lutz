pub trait Image {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn has_pixel(&self, x: usize, y: usize) -> bool;
}

#[derive(PartialEq, Eq)]
enum Marker {
    Start,
    StartOfSegment,
    EndOfSegment,
    End,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Pixel {
    pub x: usize,
    pub y: usize,
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
    start: usize,
    end: usize,
}

impl From<usize> for Range {
    fn from(start: usize) -> Self {
        Self { start, end: start }
    }
}

struct LutzObject {
    range: Option<Range>,
    info: Vec<Pixel>,
}

struct LutzState<Img, OnObject> {
    img: Img,
    on_object: OnObject,
    marker: Box<[Option<Marker>]>,
    obj_stack: Vec<LutzObject>,
    ps: PS,
    cs: CS,
    ps_stack: Vec<PS>,
    store: Box<[Vec<Pixel>]>,
}

impl<'a, Img: Image, OnObject: FnMut(Vec<Pixel>)> LutzState<&'a Img, OnObject> {
    fn new(img: &'a Img, on_object: OnObject) -> Self {
        Self {
            img,
            on_object,
            marker: std::iter::repeat_with(|| None)
                .take(img.width() + 1)
                .collect(),
            obj_stack: Vec::new(),
            ps: PS::Complete,
            cs: CS::NonObject,
            ps_stack: Vec::new(),
            store: std::iter::repeat_with(Vec::new)
                .take(img.width() + 1)
                .collect(),
        }
    }

    fn run(mut self) {
        let width = self.img.width();
        for y in 0..self.img.height() {
            self.ps = PS::Complete;
            self.cs = CS::NonObject;
            for x in 0..width {
                let newmarker = std::mem::take(&mut self.marker[x]);
                if self.img.has_pixel(x, y) {
                    // Current pixel is part of an object.
                    if self.cs != CS::Object {
                        // Previous pixel is not part of an object, start a new segment.
                        self.start_segment(x);
                    }
                    if let Some(marker) = newmarker {
                        self.process_new_marker(marker, x);
                    }
                    // Update current object by current pixel.
                    self.obj_stack.last_mut().unwrap().info.push(Pixel { x, y });
                } else {
                    // Current pixel is not part of an object.
                    if let Some(marker) = newmarker {
                        self.process_new_marker(marker, x);
                    }
                    if self.cs == CS::Object {
                        // Previous pixel was part of an object, finish segment.
                        self.end_segment(x);
                    }
                }
            }
            // Handle the extra "M+1" cell from the algorithm
            // (same logic as in the loop above, but without first branch).
            if let Some(marker) = std::mem::take(&mut self.marker[width]) {
                self.process_new_marker(marker, width);
            }
            if self.cs == CS::Object {
                self.end_segment(width);
            }
        }
    }

    fn start_segment(&mut self, x: usize) {
        self.cs = CS::Object;
        self.marker[x] = Some(if self.ps == PS::Object {
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

    fn end_segment(&mut self, x: usize) {
        self.cs = CS::NonObject;
        self.marker[x] = Some(if self.ps != PS::Complete {
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
            self.store[obj.range.unwrap().start] = obj.info;
            Marker::End
        });
    }

    fn process_new_marker(&mut self, newmarker: Marker, x: usize) {
        self.ps = match newmarker {
            Marker::Start => {
                // Start of an object on the preceding scan.
                self.ps_stack.push(self.ps);
                let store = std::mem::take(&mut self.store[x]);
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
                        self.marker[k] = Some(Marker::StartOfSegment);
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
                            (self.on_object)(obj.info);
                        }
                        Some(range) => {
                            // Object completed on this scan.
                            self.marker[range.end] = Some(Marker::End);
                            self.store[range.start] = obj.info;
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

pub fn lutz(img: &impl Image, on_object: impl FnMut(Vec<Pixel>)) {
    LutzState::new(img, on_object).run()
}
