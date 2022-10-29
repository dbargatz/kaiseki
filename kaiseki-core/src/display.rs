use anyhow::Result;

use crate::bus::{BusMessage, MessageBus, MessageBusError};
use crate::component::ComponentId;

#[derive(Clone, Debug)]
pub enum DisplayBusMessage {
    Clear,
    ClearResponse,
    DrawSprite {
        address: usize,
        length: usize,
        x_pos: usize,
        y_pos: usize,
    },
    DrawSpriteResponse {
        pixel_flipped: usize,
    },
}

impl BusMessage for DisplayBusMessage {}

pub type DisplayBus = MessageBus<DisplayBusMessage>;

impl DisplayBus {
    pub async fn clear(&self, id: &ComponentId) -> Result<(), MessageBusError> {
        let request = DisplayBusMessage::Clear;
        let response = self.request(id, request).await?;
        if let DisplayBusMessage::ClearResponse = response {
            return Ok(());
        }
        panic!("unexpected response to clear()");
    }

    pub async fn draw_sprite(
        &self,
        id: &ComponentId,
        address: u16,
        length: u8,
        x_pos: u8,
        y_pos: u8,
    ) -> Result<u8, MessageBusError> {
        let request = DisplayBusMessage::DrawSprite {
            address: address as usize,
            length: length as usize,
            x_pos: x_pos as usize,
            y_pos: y_pos as usize,
        };
        let response = self.request(id, request).await?;
        if let DisplayBusMessage::DrawSpriteResponse { pixel_flipped } = response {
            return Ok(pixel_flipped as u8);
        }
        panic!("unexpected response to draw_sprite()");
    }
}
