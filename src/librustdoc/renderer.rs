use std::io::IoResult;
use std::path::Path;

/// A renderer for a specific format.
pub trait Renderer {
    fn render(&mut self, dest: Path) -> IoResult<()>;
}
