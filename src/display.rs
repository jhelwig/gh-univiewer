use rgb::RGB8;
pub enum MetricType {
    ColumnRatio {
        width: u8,
        values: Vec<u32>,
        colors: Vec<RGB8>,
    },
    ColumnCount {
        width: u8,
        value: u32,
    },
}
