#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Matrix {
    pub width: usize,
    pub height: usize,
}

#[allow(dead_code)]
impl Matrix {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn refresh(&mut self, w: usize, h: usize) {
        self.width = w;
        self.height = h;
    }
}
