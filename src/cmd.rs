// Copyright (C) 2026 Jorge Andre Castro
// SPDX-License-Identifier: GPL-2.0-or-later
//! Constantes de commandes ST7789V (internes).
    pub(crate) const SWRESET:   u8 = 0x01;
    pub(crate) const SLPOUT:    u8 = 0x11;
    pub(crate) const NORON:     u8 = 0x13;
    pub(crate) const INVOFF:    u8 = 0x20;
    pub(crate) const INVON:     u8 = 0x21;
    pub(crate) const DISPON:    u8 = 0x29;
    pub(crate) const CASET:     u8 = 0x2A;
    pub(crate) const RASET:     u8 = 0x2B;
    pub(crate) const RAMWR:     u8 = 0x2C;
    pub(crate) const MADCTL:    u8 = 0x36;
    pub(crate) const COLMOD:    u8 = 0x3A;
    pub(crate) const PORCTRL:   u8 = 0xB2;
    pub(crate) const GCTRL:     u8 = 0xB7;
    pub(crate) const VCOMS:     u8 = 0xBB;
    pub(crate) const LCMCTRL:   u8 = 0xC0;
    pub(crate) const VDVVRHEN:  u8 = 0xC2;
    pub(crate) const VRHS:      u8 = 0xC3;
    pub(crate) const VDVS:      u8 = 0xC4;
    pub(crate) const FRCTRL2:   u8 = 0xC6;
    pub(crate) const PWCTRL1:   u8 = 0xD0;
    pub(crate) const PVGAMCTRL: u8 = 0xE0;
    pub(crate) const NVGAMCTRL: u8 = 0xE1;