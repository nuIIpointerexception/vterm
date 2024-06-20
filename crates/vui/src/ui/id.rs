#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Id(u32);

impl Id {
    pub fn new(value: u32) -> Self {
        Self(value)
    }
}

pub const fn id_hash(content: &str, line: u32, column: u32, seed: &str) -> u32 {
    let mut hash = 3581u32;
    {
        let content_bytes = content.as_bytes();
        let mut i: usize = 0;
        while i < content_bytes.len() {
            hash = hash.wrapping_mul(33).wrapping_add(content_bytes[i] as u32);
            i += 1;
        }
    }
    {
        let seed_bytes = seed.as_bytes();
        let mut j: usize = 0;
        while j < seed_bytes.len() {
            hash = hash.wrapping_mul(33).wrapping_add(seed_bytes[j] as u32);
            j += 1;
        }
    }
    hash = hash.wrapping_mul(33).wrapping_add(line);
    hash = hash.wrapping_mul(33).wrapping_add(column);
    return hash;
}

#[macro_export]
macro_rules! gen_id {
    ($str:literal) => {{
        let id: u32 = ccthw::ui::id_hash(file!(), line!(), column!(), &format!("{:?}", $str));
        ccthw::ui::Id::new(id)
    }};
    ($expr:expr) => {{
        let id: u32 = id_hash(file!(), line!(), column!(), $expr);
        Id::new(id)
    }};
    () => {{
        const ID: u32 = ccthw::ui::id_hash(file!(), line!(), column!(), "seed");
        ccthw::ui::Id::new(ID)
    }};
}
