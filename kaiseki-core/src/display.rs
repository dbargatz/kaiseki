use anyhow::Result;
use async_trait::async_trait;

use crate::bus::{Bus, BusError, BusMessage};
use crate::component::{Component, ComponentId};
use crate::memory::MemoryBus;

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

pub type DisplayBus = Bus<DisplayBusMessage>;

impl DisplayBus {
    pub async fn clear(&self, id: &ComponentId) -> Result<(), BusError> {
        let request = DisplayBusMessage::Clear;
        self.broadcast(id, request).await.unwrap();
        let (_, response) = self.recv(id).await.unwrap();

        if let DisplayBusMessage::ClearResponse = response {
            Ok(())
        } else {
            tracing::warn!(
                "unexpected response to Clear on display bus: {:?}",
                response
            );
            let msg_str = format!("{:?}", response);
            Err(BusError::UnexpectedMessage(msg_str))
        }
    }

    pub async fn draw_sprite(
        &self,
        id: &ComponentId,
        address: u16,
        length: u8,
        x_pos: u8,
        y_pos: u8,
    ) -> Result<u8, BusError> {
        let request = DisplayBusMessage::DrawSprite {
            address: address as usize,
            length: length as usize,
            x_pos: x_pos as usize,
            y_pos: y_pos as usize,
        };
        self.broadcast(id, request).await.unwrap();
        let (_, response) = self.recv(id).await.unwrap();

        if let DisplayBusMessage::DrawSpriteResponse { pixel_flipped } = response {
            Ok(pixel_flipped as u8)
        } else {
            tracing::warn!(
                "unexpected response to DrawSprite on memory bus: {:?}",
                response
            );
            let msg_str = format!("{:?}", response);
            Err(BusError::UnexpectedMessage(msg_str))
        }
    }
}

#[derive(Debug)]
pub struct MonochromeDisplay<const N: usize, const W: usize, const H: usize> {
    id: ComponentId,
    display_bus: DisplayBus,
    memory_bus: MemoryBus,
    pixels: [u8; N],
}

#[async_trait]
impl<const N: usize, const W: usize, const H: usize> Component for MonochromeDisplay<N, W, H> {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    async fn start(&mut self) {
        loop {
            let request = self.display_bus.recv(&self.id).await;
            match request {
                Ok((from, DisplayBusMessage::Clear)) => {
                    tracing::trace!("clearing display");
                    for pixel_byte in self.pixels.iter_mut() {
                        *pixel_byte = 0;
                    }
                    let response = DisplayBusMessage::ClearResponse;
                    self.display_bus
                        .send(&self.id, &from, response)
                        .await
                        .unwrap();
                }
                Ok((
                    from,
                    DisplayBusMessage::DrawSprite {
                        address,
                        length,
                        x_pos,
                        y_pos,
                    },
                )) => {
                    tracing::trace!(
                        "drawing {}-byte sprite at 0x{:04X} to ({}, {})",
                        length,
                        address,
                        x_pos,
                        y_pos
                    );
                    let sprite = self
                        .memory_bus
                        .read(&self.id, address, length)
                        .await
                        .unwrap();
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
                    self.display_bus
                        .send(&self.id, &from, response)
                        .await
                        .unwrap();
                }
                _ => panic!("unexpected request on display bus"),
            }
        }
    }
}

impl<const N: usize, const W: usize, const H: usize> MonochromeDisplay<N, W, H> {
    pub fn new(display_bus: &DisplayBus, memory_bus: &MemoryBus) -> Self {
        MonochromeDisplay {
            id: ComponentId::new("Monochrome Display"),
            display_bus: display_bus.clone(),
            memory_bus: memory_bus.clone(),
            pixels: [0; N],
        }
    }
}
