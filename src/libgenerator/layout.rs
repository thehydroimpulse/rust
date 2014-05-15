use std::path::Path;

pub struct Layout<'a> {
  name: &'a str,
  path: Path,
  contents: &'a str
}
