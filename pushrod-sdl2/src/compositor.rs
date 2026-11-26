// Compositor
// Pushrod
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The frame the [`Sdl2Backend`](crate::backend::Sdl2Backend) composes into.
//!
//! Each frame is composed into an offscreen target texture the size of the window, then copied
//! to the window when it is presented.  Keeping the frame in a texture lets a modal backdrop
//! soften everything drawn so far entirely on the GPU: the frame is scaled down by halves into
//! a chain of smaller textures and back up again, with linear filtering, which blurs it.  No
//! pixel is read or written on the CPU.

use sdl2::render::{BlendMode, Canvas, ScaleMode, Texture};
use sdl2::video::Window;

/// The most halvings a backdrop blur uses; more would only waste textures.
const MAX_BLUR: u8 = 6;

/// Makes `texture` the canvas's render target, or the window again for `None`.
///
/// The `sdl2` crate only offers render targets inside a closure, which cannot span the calls
/// that compose one frame (`begin_frame`, `draw_widget`, `present`), so the target is set
/// directly.
///
/// # Arguments
///
/// * `canvas` - The window's canvas.
/// * `texture` - A target texture created by this canvas, or `None` for the window.
///
/// # Returns
///
/// `Ok(())` once the target is set.
///
/// # Errors
///
/// Returns the `SDL2` message if the renderer refuses the target.
fn set_target(canvas: &mut Canvas<Window>, texture: Option<&Texture>) -> Result<(), String> {
    let raw = texture.map_or(std::ptr::null_mut(), |t| t.raw());

    // SAFETY: the renderer is alive for as long as `canvas`, and `texture` (when given) was
    // created by it with target access and has not been destroyed.
    let result = unsafe { sdl2::sys::SDL_SetRenderTarget(canvas.raw(), raw) };

    if result == 0 {
        Ok(())
    } else {
        Err(sdl2::get_error())
    }
}

/// Creates a target texture that scales with linear filtering and copies without blending.
///
/// # Arguments
///
/// * `canvas` - The window's canvas.
/// * `w` - The width in physical pixels; at least `1`.
/// * `h` - The height in physical pixels; at least `1`.
///
/// # Returns
///
/// The texture.
///
/// # Errors
///
/// Returns the `SDL2` message if the texture cannot be created.
fn create_target(canvas: &mut Canvas<Window>, w: u32, h: u32) -> Result<Texture, String> {
    let mut texture = canvas
        .create_texture_target(None, w.max(1), h.max(1))
        .map_err(|e| e.to_string())?;

    texture.set_scale_mode(ScaleMode::Linear);
    texture.set_blend_mode(BlendMode::None);

    Ok(texture)
}

/// Destroys every texture in `textures`.
///
/// # Arguments
///
/// * `textures` - Textures created by the canvas that is still alive.
fn destroy_all(textures: impl IntoIterator<Item = Texture>) {
    for texture in textures {
        // SAFETY: with `unsafe_textures` the renderer must outlive its textures; the compositor
        // is declared before the canvas in `Sdl2Backend`, so the renderer is still alive.
        unsafe { texture.destroy() };
    }
}

/// The offscreen frame and the textures its backdrop blur uses.
#[derive(Default)]
pub(crate) struct Compositor {
    /// The frame being composed, the size of the window's drawing area.
    frame: Option<Texture>,
    /// The frame's size in physical pixels.
    size: (u32, u32),
    /// Successively halved copies of the frame, for the blur: half, quarter, and so on.
    chain: Vec<Texture>,
    /// `true` between `begin` and `present`, while the frame texture is the render target.
    active: bool,
}

impl Compositor {
    /// Starts a frame: makes the offscreen frame (resized to the window if needed) the render
    /// target.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The window's canvas.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the frame is the target.
    ///
    /// # Errors
    ///
    /// Returns the `SDL2` message if the frame cannot be created or targeted; the caller then
    /// draws straight to the window.
    pub(crate) fn begin(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let size = canvas.output_size()?;

        if self.frame.is_none() || self.size != size {
            destroy_all(self.frame.take());
            destroy_all(self.chain.drain(..));
            self.frame = Some(create_target(canvas, size.0, size.1)?);
            self.size = size;
        }

        set_target(canvas, self.frame.as_ref())?;
        self.active = true;

        Ok(())
    }

    /// Softens and darkens everything in the frame so far.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The window's canvas.
    /// * `shade` - The color blended over the frame.
    /// * `blur` - How many halvings to blur through; `0` skips the blur.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the backdrop is drawn.
    ///
    /// # Errors
    ///
    /// Returns the `SDL2` message if a texture cannot be created or drawn.
    pub(crate) fn backdrop(
        &mut self,
        canvas: &mut Canvas<Window>,
        shade: sdl2::pixels::Color,
        blur: u8,
    ) -> Result<(), String> {
        if self.active && blur > 0 {
            self.blur(canvas, blur.min(MAX_BLUR) as usize)?;
        }

        if shade.a > 0 {
            canvas.set_blend_mode(BlendMode::Blend);
            canvas.set_draw_color(shade);

            let filled = canvas.fill_rect(None);

            canvas.set_blend_mode(BlendMode::None);
            filled?;
        }

        Ok(())
    }

    /// Blurs the frame by scaling it down `levels` times by half, then back up, each step with
    /// linear filtering.  Leaves the frame as the render target.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The window's canvas.
    /// * `levels` - How many halvings; at least `1`.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the frame holds the blurred image.
    ///
    /// # Errors
    ///
    /// Returns the `SDL2` message if a texture cannot be created or drawn.
    fn blur(&mut self, canvas: &mut Canvas<Window>, levels: usize) -> Result<(), String> {
        while self.chain.len() < levels {
            let shift = self.chain.len() + 1;
            let texture = create_target(canvas, self.size.0 >> shift, self.size.1 >> shift)?;

            self.chain.push(texture);
        }

        let Some(frame) = self.frame.as_ref() else {
            return Ok(());
        };

        // Down: frame -> half -> quarter -> ...
        for i in 0..levels {
            let source = if i == 0 { frame } else { &self.chain[i - 1] };

            set_target(canvas, Some(&self.chain[i]))?;
            canvas.copy(source, None, None)?;
        }

        // Up: ... -> quarter -> half -> frame.
        for i in (0..levels - 1).rev() {
            set_target(canvas, Some(&self.chain[i]))?;
            canvas.copy(&self.chain[i + 1], None, None)?;
        }

        set_target(canvas, Some(frame))?;
        canvas.copy(&self.chain[0], None, None)
    }

    /// Ends the frame: copies it to the window.  The caller presents the window afterward.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The window's canvas.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the frame is on the window, or if no frame was being composed.
    ///
    /// # Errors
    ///
    /// Returns the `SDL2` message if the copy fails.
    pub(crate) fn finish(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        if !self.active {
            return Ok(());
        }

        self.active = false;
        set_target(canvas, None)?;

        match self.frame.as_ref() {
            Some(frame) => canvas.copy(frame, None, None),
            None => Ok(()),
        }
    }

    /// Runs `read` with the last composed frame as the render target, so it can read the
    /// frame's pixels, then restores the target.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The window's canvas.
    /// * `read` - Reads from the canvas.
    ///
    /// # Returns
    ///
    /// What `read` returned.
    ///
    /// # Errors
    ///
    /// Returns the `SDL2` message if the target cannot be changed, or `read`'s error.
    pub(crate) fn read_frame<T>(
        &mut self,
        canvas: &mut Canvas<Window>,
        read: impl FnOnce(&mut Canvas<Window>) -> Result<T, String>,
    ) -> Result<T, String> {
        if self.frame.is_none() {
            return read(canvas);
        }

        set_target(canvas, self.frame.as_ref())?;

        let result = read(canvas);

        if !self.active {
            set_target(canvas, None)?;
        }

        result
    }
}

impl Drop for Compositor {
    /// Frees the frame and blur textures (`unsafe_textures` textures are not freed on drop).
    fn drop(&mut self) {
        destroy_all(self.frame.take());
        destroy_all(self.chain.drain(..));
    }
}
