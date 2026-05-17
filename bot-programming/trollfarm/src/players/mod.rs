mod internal;
use internal::{print_internal};

pub struct Test {}

impl Test {
    pub fn testing() {
        print_internal();
        eprintln!("dafsadfsadfsad");
    }
}
