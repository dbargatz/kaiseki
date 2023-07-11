use anyhow::Result;

use crate::component::ExecutableComponent;

pub trait Machine: ExecutableComponent {
    fn get_frame(&self) -> (usize, usize, Vec<u8>);
    fn load(&self, file: &str) -> Result<()>;
}
