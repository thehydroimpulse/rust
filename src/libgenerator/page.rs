use std::path::Path;

pub struct Page<'a> {
  path: Path,
  contents: &'a str
}
