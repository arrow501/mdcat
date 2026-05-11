// Copyright Sebastian Wiesner <sebastian@swsnr.de>
// Copyright arrow.swiech@gmail.com
//
// This file is part of a fork of mdcat and was not authored by Sebastian Wiesner.
// Sebastian Wiesner is not affiliated with these modifications or their use of AI assistance.

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Runtime terminal capability detection via escape sequence queries.
//!
//! Probes the terminal directly to discover the best supported image protocol,
//! independent of `$TERM` or `$TERM_PROGRAM`.

use crate::terminal::capabilities::{
    ImageCapability,
    kitty::KittyGraphicsProtocol,
    sixel::SixelProtocol,
};

#[cfg(unix)]
mod unix {
    use std::fs::OpenOptions;
    use std::io::{Read, Write};

    use rustix::termios::{
        tcgetattr, tcsetattr, InputModes, LocalModes, OptionalActions, SpecialCodeIndex,
    };

    use super::ImageCapability;
    use super::{KittyGraphicsProtocol, SixelProtocol};

    /// Read bytes from `tty` until `terminator` is seen or timeout/error.
    fn read_until(tty: &mut std::fs::File, terminator: u8, max: usize) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);
        let mut byte = [0u8; 1];
        for _ in 0..max {
            match tty.read(&mut byte) {
                Ok(1) => {
                    buf.push(byte[0]);
                    if byte[0] == terminator {
                        break;
                    }
                }
                _ => break,
            }
        }
        buf
    }

    /// Read until ESC \ (ST, String Terminator).
    fn read_until_st(tty: &mut std::fs::File) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);
        let mut byte = [0u8; 1];
        let mut prev = 0u8;
        for _ in 0..512 {
            match tty.read(&mut byte) {
                Ok(1) => {
                    buf.push(byte[0]);
                    if prev == 0x1b && byte[0] == b'\\' {
                        break;
                    }
                    prev = byte[0];
                }
                _ => break,
            }
        }
        buf
    }

    /// Send a kitty graphics query and return true if the terminal responds.
    fn try_kitty(tty: &mut std::fs::File) -> bool {
        let _ = tty.write_all(b"\x1b_Ga=q,i=31,s=1,v=1;\x1b\\");
        let _ = tty.flush();
        let response = read_until_st(tty);
        // A kitty response contains the APC introducer _G
        response.windows(2).any(|w| w == b"_G")
    }

    /// Send DA1 and return true if the response includes parameter 4 (sixel).
    fn try_sixel(tty: &mut std::fs::File) -> bool {
        let _ = tty.write_all(b"\x1b[c");
        let _ = tty.flush();
        // DA1 response: ESC [ ? p1 ; p2 ; ... c
        let response = read_until(tty, b'c', 256);
        let s = String::from_utf8_lossy(&response);
        // Sixel support is parameter "4" anywhere in the list
        s.split(|c: char| c == ';' || c == '?' || c == '[')
            .any(|part| part.trim() == "4")
    }

    pub fn detect_image_capability() -> Option<ImageCapability> {
        let mut tty = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .ok()?;

        let orig = tcgetattr(&tty).ok()?;
        let mut raw = orig.clone();

        // Raw mode: disable echo, canonical processing, and signals.
        // VMIN=0 + VTIME=2: each read waits up to 200 ms, returns 0 if no data.
        raw.local_modes &= !(LocalModes::ICANON | LocalModes::ECHO | LocalModes::ISIG);
        raw.input_modes &= !(InputModes::IXON | InputModes::ICRNL);
        raw.special_codes[SpecialCodeIndex::VMIN] = 0;
        raw.special_codes[SpecialCodeIndex::VTIME] = 2;

        tcsetattr(&tty, OptionalActions::Drain, &raw).ok()?;

        let capability = if try_kitty(&mut tty) {
            Some(ImageCapability::Kitty(KittyGraphicsProtocol))
        } else if try_sixel(&mut tty) {
            Some(ImageCapability::Sixel(SixelProtocol))
        } else {
            None
        };

        // Always restore original terminal state.
        let _ = tcsetattr(&tty, OptionalActions::Drain, &orig);

        capability
    }
}

/// Detect the best image capability by querying the terminal directly.
///
/// Probes for kitty graphics protocol first, then sixel via DA1.
/// Returns `None` when detection fails or no image protocol is supported.
///
/// Only meaningful when output goes to a real terminal. Callers should skip
/// this when paginating or using `--ansi`.
#[cfg(unix)]
pub fn detect_image_capability() -> Option<ImageCapability> {
    unix::detect_image_capability()
}

#[cfg(not(unix))]
pub fn detect_image_capability() -> Option<ImageCapability> {
    None
}
