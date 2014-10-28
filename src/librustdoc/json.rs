use renderer::Renderer;
use clean::Crate;
use std::path::Path;

pub struct JsonRenderer {
    krate: clean::Crate
}

impl JsonRenderer {
    pub fn new(krate: clean::Crate) -> JsonRenderer {
        JsonRenderer {
            krate: krate
        }
    }
}

impl Renderer for JsonRenderer {
    fn render(&mut self, dest: Path) -> IoResult<()> {
        Ok(())
    }
}
