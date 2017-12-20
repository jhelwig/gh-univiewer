pub enum MetricType {
    ColumnRatio {
        width: u8,
        values: Vec<u32>,
        colors: Vec<RGB>,
    },
    ColumnCount {
        width: u8,
        value: u32,
    },
}

#[derive(Copy, Clone)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { r: red, g: green, b: blue }
    }
}

