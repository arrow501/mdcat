// Copyright Sebastian Wiesner <sebastian@swsnr.de>
// Copyright arrow.swiech@gmail.com
//
// This file is part of a fork of mdcat and was not authored by Sebastian Wiesner.
// Sebastian Wiesner is not affiliated with these modifications or their use of AI assistance.

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Sixel terminal image protocol.

use std::io::{Result, Write};

use tracing::{event, instrument, Level};

use crate::resources::image::{downsize_to_columns, InlineImageProtocol};
use crate::resources::MimeData;
use crate::terminal::size::TerminalSize;

/// Sixel terminal image protocol.
#[derive(Debug, Copy, Clone)]
pub struct SixelProtocol;

#[cfg(feature = "image-processing")]
fn quantize(image: &image::RgbImage, num_colors: usize) -> (Vec<u8>, Vec<u8>) {
    // NeuQuant requires 4-channel RGBA input
    let pixels: Vec<u8> = image
        .pixels()
        .flat_map(|p| [p.0[0], p.0[1], p.0[2], 255])
        .collect();
    let nq = color_quant::NeuQuant::new(10, num_colors, &pixels);
    let indices: Vec<u8> = image
        .pixels()
        .map(|p| nq.index_of(&[p.0[0], p.0[1], p.0[2], 255]) as u8)
        .collect();
    (indices, nq.color_map_rgb())
}

#[cfg(feature = "image-processing")]
fn rle_encode(sixels: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    if sixels.is_empty() {
        return out;
    }
    let mut run = 1u32;
    let mut cur = sixels[0];
    for &b in &sixels[1..] {
        if b == cur {
            run += 1;
        } else {
            push_run(&mut out, run, cur);
            cur = b;
            run = 1;
        }
    }
    push_run(&mut out, run, cur);
    out
}

#[cfg(feature = "image-processing")]
fn push_run(out: &mut Vec<u8>, run: u32, ch: u8) {
    if run > 3 {
        write!(out, "!{run}").unwrap();
        out.push(ch);
    } else {
        for _ in 0..run {
            out.push(ch);
        }
    }
}

#[cfg(feature = "image-processing")]
fn encode_sixel(image: &image::RgbImage) -> Vec<u8> {
    
    let (width, height) = image.dimensions();
    let w = width as usize;
    let h = height as usize;

    let (indices, palette) = quantize(image, 256);
    let actual_colors = palette.len() / 3;

    let mut out: Vec<u8> = Vec::new();

    // DCS: P1=0 (aspect ratio default), P2=1 (background transparent), P3=8 (grid size)
    out.extend_from_slice(b"\x1bP0;1;8q");

    // Define all color registers up front
    for i in 0..actual_colors {
        let r = palette[i * 3] as u32 * 100 / 255;
        let g = palette[i * 3 + 1] as u32 * 100 / 255;
        let b = palette[i * 3 + 2] as u32 * 100 / 255;
        write!(out, "#{i};2;{r};{g};{b}").unwrap();
    }

    let num_bands = (h + 5) / 6;

    for band in 0..num_bands {
        let y_start = band * 6;
        let mut first_color_in_band = true;

        for c in 0..actual_colors {
            let mut sixels: Vec<u8> = Vec::with_capacity(w);
            let mut has_pixels = false;

            for x in 0..w {
                let mut mask: u8 = 0;
                for dy in 0..6 {
                    let y = y_start + dy;
                    if y < h && indices[y * w + x] as usize == c {
                        mask |= 1 << dy;
                    }
                }
                if mask != 0 {
                    has_pixels = true;
                }
                sixels.push(mask + 0x3F);
            }

            if !has_pixels {
                continue;
            }

            if !first_color_in_band {
                out.push(b'$'); // CR: return to start of line
            }
            first_color_in_band = false;

            write!(out, "#{c}").unwrap();
            out.extend_from_slice(&rle_encode(&sixels));
        }

        out.push(b'-'); // DECGNL: advance to next sixel band
    }

    out.extend_from_slice(b"\x1b\\"); // ST
    out
}

impl SixelProtocol {
    #[cfg(feature = "image-processing")]
    fn render(self, mime_data: MimeData, terminal_size: TerminalSize) -> Result<Vec<u8>> {
        use image::ImageFormat;
        use std::io::{Error, ErrorKind};

        let image = if let Some("image/svg+xml") = mime_data.mime_type_essence() {
            let png_data = crate::resources::svg::render_svg_to_png(&mime_data.data)?;
            image::load_from_memory_with_format(&png_data, ImageFormat::Png)
                .map_err(|e| Error::new(ErrorKind::Other, e))?
        } else {
            let fmt = mime_data
                .mime_type_essence()
                .and_then(image::ImageFormat::from_mime_type);
            match fmt {
                Some(f) => image::load_from_memory_with_format(&mime_data.data, f)
                    .map_err(|e| Error::new(ErrorKind::Other, e))?,
                None => image::load_from_memory(&mime_data.data)
                    .map_err(|e| Error::new(ErrorKind::Other, e))?,
            }
        };

        let image = match downsize_to_columns(&image, terminal_size) {
            Some(resized) => resized,
            None => image,
        };

        Ok(encode_sixel(&image.into_rgb8()))
    }
}

impl InlineImageProtocol for SixelProtocol {
    #[instrument(skip(self, writer, resource_handler, terminal_size), fields(url = %url))]
    fn write_inline_image(
        &self,
        writer: &mut dyn Write,
        resource_handler: &dyn crate::ResourceUrlHandler,
        url: &url::Url,
        terminal_size: TerminalSize,
    ) -> Result<()> {
        let mime_data = resource_handler.read_resource(url)?;
        event!(
            Level::DEBUG,
            "Received data of mime type {:?}",
            mime_data.mime_type
        );

        #[cfg(feature = "image-processing")]
        {
            let sixel_data = self.render(mime_data, terminal_size)?;
            writer.write_all(&sixel_data)?;
        }

        #[cfg(not(feature = "image-processing"))]
        {
            let _ = (mime_data, terminal_size);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "sixel requires the image-processing feature",
            ));
        }

        Ok(())
    }
}
