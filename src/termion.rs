// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! Conversion methods for [`termion`][]’s text style types.
//!
//! *Requires the `termion` feature.*
//!
//! Termion does not use store the text format in generic data types together with the formatted
//! texts.  Instead, if provides separate commands for all formatting options.  These commands
//! produce ANSI escape sequencens that can be printed to stdout.
//!
//! This module defines the [`Termion`][] trait that is implemented for [`StyledStr`][] and
//! [`StyledString`][].  Its [`termion`][`Termion::termion`] method produces an instance of the
//! [`TermionStr`][] struct that can be converted into the escape sequences produced by `termion`
//! using its [`Display`][] implementation.
//!
//! Alternatively, you can use the [`render`][] function to render a single string and the
//! [`render_iter`][] function to render an iterator over strings.
//!
//! Note that this implementation always uses [`termion::style::Reset`][] to clear the formatting
//! instead of [`termion::style::NoBold`][] etc. for compatibility with terminals that don’t
//! support the *No Bold* style.
//!
//! # Examples
//!
//! Rendering a single string:
//!
//! ```
//! let s = text_style::StyledStr::plain("test").bold();
//! text_style::termion::render(std::io::stdout(), s)
//!     .expect("Failed to render string");
//! ```
//!
//! Rendering multiple strings:
//!
//! ```
//! let v = vec![
//!     text_style::StyledStr::plain("test").bold(),
//!     text_style::StyledStr::plain(" "),
//!     text_style::StyledStr::plain("test2").italic(),
//! ];
//! text_style::termion::render_iter(std::io::stdout(), v.iter())
//!     .expect("Failed to render string");
//! ```
//!
//! Using the [`Termion`][] trait:
//!
//! ```
//! use text_style::termion::Termion;
//!
//! println!("{}", text_style::StyledStr::plain("test").bold().termion());
//! ```
//!
//! [`Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
//! [`termion`]: https://docs.rs/termion
//! [`termion::style::Reset`]: https://docs.rs/termion/latest/termion/style/struct.Reset.html
//! [`termion::style::NoBold`]: https://docs.rs/termion/latest/termion/style/struct.NoBold.html
//! [`StyledStr`]: ../struct.StyledStr.html
//! [`StyledString`]: ../struct.StyledString.html
//! [`render`]: fn.render.html
//! [`render_iter`]: fn.render_iter.html
//! [`Termion`]: trait.Termion.html
//! [`Termion::termion`]: trait.Termion.html#tymethod.termion
//! [`TermionStr`]: struct.TermionStr.html

use std::borrow;
use std::fmt;
use std::io;

use termion::{color, style};

use crate::{AnsiColor, AnsiMode, Color, Effect, Style, StyledStr, StyledString};

/// A styled string that can be rendered using `termion`.
///
/// The [`Display`][] implementation of this struct produces a formatted string using the escape
/// sequencens generated by termion.
///
/// # Example
///
/// ```
/// use text_style::termion::Termion;
///
/// println!("{}", text_style::StyledStr::plain("test").bold().termion());
/// ```
pub struct TermionStr<'a> {
    s: &'a str,
    style: Option<Style>,
}

impl<'a> fmt::Display for TermionStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(style) = &self.style {
            if let Some(fg) = style.fg {
                f.write_str(get_fg(fg).as_ref())?;
            }
            if let Some(bg) = style.bg {
                f.write_str(get_bg(bg).as_ref())?;
            }
            for effect in style.effects.iter() {
                f.write_str(get_effect(effect))?;
            }
        }
        f.write_str(self.s)?;
        if let Some(style) = &self.style {
            if style.fg.is_some() || style.bg.is_some() || !style.effects.is_empty() {
                f.write_str(style::Reset.as_ref())?;
            }
        }
        Ok(())
    }
}

/// Extension trait for producing formatted strings with `termion`.
///
/// # Example
///
/// ```
/// use text_style::termion::Termion;
///
/// println!("{}", text_style::StyledStr::plain("test").bold().termion());
/// ```
pub trait Termion {
    /// Convert this string into a [`TermionStr`][] that can be formatted with `termion`.
    ///
    /// # Example
    ///
    /// ```
    /// use text_style::termion::Termion;
    ///
    /// println!("{}", text_style::StyledStr::plain("test").bold().termion());
    /// ```
    ///
    /// [`TermionStr`]: struct.TermionStr.html
    fn termion(&self) -> TermionStr<'_>;
}

impl<'a> Termion for StyledStr<'a> {
    fn termion(&self) -> TermionStr<'_> {
        TermionStr {
            s: self.s,
            style: self.style,
        }
    }
}

impl Termion for StyledString {
    fn termion(&self) -> TermionStr<'_> {
        TermionStr {
            s: &self.s,
            style: self.style,
        }
    }
}

fn get_bg(color: Color) -> borrow::Cow<'static, str> {
    match color {
        Color::Ansi { color, mode } => get_ansi_bg(color, mode).into(),
        Color::Rgb { r, g, b } => color::Rgb(r, g, b).bg_string().into(),
    }
}

fn get_ansi_bg(color: AnsiColor, mode: AnsiMode) -> &'static str {
    use AnsiColor::*;
    use AnsiMode::*;

    match (mode, color) {
        (Dark, Black) => color::Black.bg_str(),
        (Dark, Red) => color::Red.bg_str(),
        (Dark, Green) => color::Green.bg_str(),
        (Dark, Yellow) => color::Yellow.bg_str(),
        (Dark, Blue) => color::Blue.bg_str(),
        (Dark, Magenta) => color::Magenta.bg_str(),
        (Dark, Cyan) => color::Cyan.bg_str(),
        (Dark, White) => color::White.bg_str(),
        (Light, Black) => color::LightBlack.bg_str(),
        (Light, Red) => color::LightRed.bg_str(),
        (Light, Green) => color::LightGreen.bg_str(),
        (Light, Yellow) => color::LightYellow.bg_str(),
        (Light, Blue) => color::LightBlue.bg_str(),
        (Light, Magenta) => color::LightMagenta.bg_str(),
        (Light, Cyan) => color::LightCyan.bg_str(),
        (Light, White) => color::LightWhite.bg_str(),
    }
}

fn get_fg(color: Color) -> borrow::Cow<'static, str> {
    match color {
        Color::Ansi { color, mode } => get_ansi_fg(color, mode).into(),
        Color::Rgb { r, g, b } => color::Rgb(r, g, b).fg_string().into(),
    }
}

fn get_ansi_fg(color: AnsiColor, mode: AnsiMode) -> &'static str {
    use AnsiColor::*;
    use AnsiMode::*;

    match (mode, color) {
        (Dark, Black) => color::Black.fg_str(),
        (Dark, Red) => color::Red.fg_str(),
        (Dark, Green) => color::Green.fg_str(),
        (Dark, Yellow) => color::Yellow.fg_str(),
        (Dark, Blue) => color::Blue.fg_str(),
        (Dark, Magenta) => color::Magenta.fg_str(),
        (Dark, Cyan) => color::Cyan.fg_str(),
        (Dark, White) => color::White.fg_str(),
        (Light, Black) => color::LightBlack.fg_str(),
        (Light, Red) => color::LightRed.fg_str(),
        (Light, Green) => color::LightGreen.fg_str(),
        (Light, Yellow) => color::LightYellow.fg_str(),
        (Light, Blue) => color::LightBlue.fg_str(),
        (Light, Magenta) => color::LightMagenta.fg_str(),
        (Light, Cyan) => color::LightCyan.fg_str(),
        (Light, White) => color::LightWhite.fg_str(),
    }
}

fn get_effect(effect: Effect) -> &'static str {
    match effect {
        Effect::Bold => style::Bold.as_ref(),
        Effect::Italic => style::Italic.as_ref(),
        Effect::Underline => style::Underline.as_ref(),
    }
}

/// Renders a styled string to the given output using `termion`.
///
/// # Example
///
/// ```
/// let s = text_style::StyledStr::plain("test").bold();
/// text_style::termion::render(std::io::stdout(), s)
///     .expect("Failed to render string");
/// ```
pub fn render<'a>(mut w: impl io::Write, s: impl Into<StyledStr<'a>>) -> io::Result<()> {
    write!(w, "{}", s.into().termion())
}

/// Renders multiple styled string to the given output using `termion`.
///
/// # Example
///
/// ```
/// let v = vec![
///     text_style::StyledStr::plain("test").bold(),
///     text_style::StyledStr::plain(" "),
///     text_style::StyledStr::plain("test2").italic(),
/// ];
/// text_style::termion::render_iter(std::io::stdout(), v.iter())
///     .expect("Failed to render string");
/// ```
pub fn render_iter<'a, I, Iter, S, W>(mut w: W, iter: I) -> io::Result<()>
where
    I: IntoIterator<Item = S, IntoIter = Iter>,
    Iter: Iterator<Item = S>,
    S: Into<StyledStr<'a>>,
    W: io::Write,
{
    for s in iter {
        write!(w, "{}", s.into().termion())?;
    }
    Ok(())
}
