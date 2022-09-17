use anyhow::Result;

use crate::component::ExecutableComponent;

pub trait Machine: ExecutableComponent {
    fn load(&self, file: &str) -> Result<()>;
}
