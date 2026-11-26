// Texture Store
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

//! # TextureStore
//!
//! The `TextureStore` is used to store the texture on a GPU that can be used as a drawing
//! canvas object.
//!
//! The `Canvas` and `Texture` refer to the `sdl2::render` crate.  Please visit that crate for
//! more information on the drawing features available.

use pushrod::error::Error;
use pushrod::geometry::Size;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

/// Holds the GPU texture a `Widget` paints into.
///
/// The [`Sdl2Backend`](crate::Sdl2Backend) keeps one per `Widget`.  The texture is created
/// the first time the `Widget` is painted and recreated whenever its size changes.
#[derive(Default)]
pub struct TextureStore {
    store: Option<Texture>,
    size: Size,
    invalidated: bool,
}

impl std::fmt::Debug for TextureStore {
    /// Formats the value for debugging.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextureStore").finish_non_exhaustive()
    }
}

impl TextureStore {
    /// Returns the stored texture for drawing into.
    ///
    /// # Returns
    ///
    /// The texture.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Texture`] if no texture has been created yet (see
    /// [`create_or_resize_texture`](TextureStore::create_or_resize_texture)).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// c.with_texture_canvas(texture_store.get_mut_ref()?, |texture| {
    ///     texture.set_draw_color(base_color);
    ///     texture.clear();
    /// })?;
    /// ```
    pub fn get_mut_ref(&mut self) -> Result<&mut Texture, Error> {
        self.store
            .as_mut()
            .ok_or_else(|| Error::texture("no texture has been created yet"))
    }

    /// Returns the stored texture, if one has been created.
    ///
    /// # Returns
    ///
    /// The texture, or `None` if [`create_or_resize_texture`](TextureStore::create_or_resize_texture)
    /// has not been called yet.
    pub fn get_optional_ref(&mut self) -> Option<&Texture> {
        self.store.as_ref()
    }

    /// Returns the size of the stored texture.
    ///
    /// # Returns
    ///
    /// The width and height in physical pixels, as last given to
    /// [`create_or_resize_texture`](TextureStore::create_or_resize_texture); zero before then.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Makes sure a texture of the given size exists.
    ///
    /// Creates a texture if there is none, or replaces it if the size changed (destroying the old one,
    /// since `unsafe_textures` textures aren't freed when dropped).  A new texture marks the store as
    /// invalidated.  Does nothing if a texture of this size already exists.
    ///
    /// # Arguments
    ///
    /// * `c` - The canvas whose renderer creates the texture.
    /// * `size` - The required width and height in pixels.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Texture`] if `SDL2` cannot create the texture.  The store then holds no
    /// texture, and the next call tries again.
    pub fn create_or_resize_texture(
        &mut self,
        c: &mut Canvas<Window>,
        size: Size,
    ) -> Result<(), Error> {
        if self.store.is_none() || self.size.w != size.w || self.size.h != size.h {
            self.size.w = size.w;
            self.size.h = size.h;

            // With the `unsafe_textures` feature, textures are not freed when dropped, so the
            // previous texture must be destroyed explicitly while the renderer is still alive.
            if let Some(old_texture) = self.store.take() {
                unsafe { old_texture.destroy() };
            }

            self.store = Some(
                c.create_texture_target(None, size.w, size.h)
                    .map_err(|e| Error::texture(e.to_string()))?,
            );

            eprintln!(
                "[create_or_resize_texture] Created texture: size={}x{} (memory={})",
                size.w,
                size.h,
                size.get_memory_size()
            );

            self.set_invalidated(true);
        }

        Ok(())
    }

    /// Removes the texture from the store, so it can be drawn into while the owning `Widget` is
    /// borrowed for painting.
    ///
    /// Must be followed by [`restore`](TextureStore::restore).
    ///
    /// # Returns
    ///
    /// The texture, or `None` if none has been created.
    pub(crate) fn take(&mut self) -> Option<Texture> {
        self.store.take()
    }

    /// Puts back a texture removed with [`take`](TextureStore::take).
    ///
    /// # Arguments
    ///
    /// * `texture` - The texture to store.
    pub(crate) fn restore(&mut self, texture: Texture) {
        self.store = Some(texture);
    }

    /// Returns whether the texture has been recreated and needs to be redrawn.
    ///
    /// # Returns
    ///
    /// `true` if the texture needs to be redrawn.
    pub fn is_invalidated(&self) -> bool {
        self.invalidated
    }

    /// Sets whether the texture needs to be redrawn.
    ///
    /// # Arguments
    ///
    /// * `state` - `true` to mark the texture as needing a redraw.
    pub fn set_invalidated(&mut self, state: bool) {
        self.invalidated = state;
    }
}
