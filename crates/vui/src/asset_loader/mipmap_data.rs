#[derive(Clone, Debug)]
pub struct MipmapData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl MipmapData {
    pub fn allocate(width: u32, height: u32, initial_value: [u8; 4]) -> Self {
        let mut data = vec![0; (width * height * 4) as usize];
        for i in 0..(width * height) {
            data[(i * 4 + 0) as usize] = initial_value[0];
            data[(i * 4 + 1) as usize] = initial_value[1];
            data[(i * 4 + 2) as usize] = initial_value[2];
            data[(i * 4 + 3) as usize] = initial_value[3];
        }
        Self {
            width,
            height,
            data,
        }
    }

    pub fn write_pixel(&mut self, x: u32, y: u32, value: [u8; 4]) {
        let index = ((x + y * self.width) * 4) as usize;
        for i in 0..4 {
            self.data[index + i] = value[i];
        }
    }
}
