// Copyright (C) 2026 RM4 LLC
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() -> std::process::ExitCode {
    std::process::ExitCode::from(rm4dev::run(std::env::args()) as u8)
}
