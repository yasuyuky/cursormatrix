use crate::core::{TermSize, Tty};
use std::io::Error;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Matrix {
    pub size: TermSize,
}

#[allow(dead_code)]
impl Matrix {
    pub fn from_tty(tty: &Tty) -> Result<Matrix, Error> {
        let mut matrix = Matrix { size: TermSize::from_tty(tty)? };
        matrix.refresh()?;
        Ok(matrix)
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        self.size.refresh()
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        self.refresh()
    }
}
