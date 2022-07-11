// New type wrapper around Proj which implements `Send`
pub(crate) struct Proj(pub(crate) proj::Proj);

impl Proj {
    pub(crate) fn new(from: &str, to: &str) -> Self {
        Proj(proj::Proj::new_known_crs(from, to, None).unwrap())
    }
}

unsafe impl Send for Proj {}
