pub mod projections {
    use crate::Mat4;

    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
        let mh = 2.0 / (right - left);
        let bh = (right + left) / (left - right);
        let mv = 2.0 / (bottom - top);
        let bv = (top + bottom) / (top - bottom);
        let mz = 1.0 / (far - near);
        let bz = near / (near - far);
        Mat4::new(mh, 0.0, 0.0, bh, 0.0, mv, 0.0, bv, 0.0, 0.0, mz, bz, 0.0, 0.0, 0.0, 1.0)
    }
}
