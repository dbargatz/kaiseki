use anyhow::Result;
use async_trait::async_trait;

use crate::bus::{AddressableBus, BusMessage, MessageBus, MessageBusError};
use crate::component::{Component, ComponentId, ExecutableComponent};

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

#[derive(Debug)]
pub struct MonochromeDisplay<const N: usize, const W: usize, const H: usize> {
    id: ComponentId,
    display_bus: DisplayBus,
    memory_bus: AddressableBus,
    pixels: [u8; N],
}

impl<const N: usize, const W: usize, const H: usize> Component for MonochromeDisplay<N, W, H> {
    fn id(&self) -> &ComponentId {
        &self.id
    }
}

#[async_trait]
impl<const N: usize, const W: usize, const H: usize> ExecutableComponent
    for MonochromeDisplay<N, W, H>
{
    async fn start(&mut self) {
        loop {
            let request = self.display_bus.recv(&self.id).await;
            match request {
                Ok((DisplayBusMessage::Clear, responder)) => {
                    tracing::trace!("clearing display");
                    for pixel_byte in self.pixels.iter_mut() {
                        *pixel_byte = 0;
                    }
                    let response = DisplayBusMessage::ClearResponse;
                    responder.unwrap().send(response).unwrap();
                }
                Ok((
                    DisplayBusMessage::DrawSprite {
                        address,
                        length,
                        x_pos,
                        y_pos,
                    },
                    responder,
                )) => {
                    tracing::trace!(
                        "drawing {}-byte sprite at 0x{:04X} to ({}, {})",
                        length,
                        address,
                        x_pos,
                        y_pos
                    );
                    let sprite = self.memory_bus.read(address, length).unwrap();
                    let mut pixel_flipped = 0;
                    for (sprite_row, sprite_byte) in sprite.iter().enumerate() {
                        let pixel_y_idx = (y_pos + sprite_row) * W;
                        for sprite_col in 0..=7 {
                            let pixel_x_idx = x_pos + sprite_col;
                            let pixel_byte_idx = (pixel_y_idx + pixel_x_idx) / 8;
                            let pixel_bit_idx = (pixel_y_idx + pixel_x_idx) % 8;

                            let sprite_bit =
                                (sprite_byte & (0x80 >> sprite_col)) >> (7 - sprite_col);
                            let sprite_mask = sprite_bit << (7 - pixel_bit_idx);
                            let pixel_byte = self.pixels.get_mut(pixel_byte_idx).unwrap();
                            let prev_value = *pixel_byte;
                            *pixel_byte ^= sprite_mask;
                            if prev_value != *pixel_byte {
                                pixel_flipped = 1;
                            }
                        }
                    }
                    let response = DisplayBusMessage::DrawSpriteResponse { pixel_flipped };
                    responder.unwrap().send(response).unwrap();
                }
                _ => panic!("unexpected request on display bus"),
            }
        }
    }
}

impl<const N: usize, const W: usize, const H: usize> MonochromeDisplay<N, W, H> {
    pub fn new(display_bus: &DisplayBus, memory_bus: &AddressableBus) -> Self {
        MonochromeDisplay {
            id: ComponentId::new("Monochrome Display"),
            display_bus: display_bus.clone(),
            memory_bus: memory_bus.clone(),
            pixels: [0; N],
        }
    }
}
