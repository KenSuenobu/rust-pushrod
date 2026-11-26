// Image Cache
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

//! # ImageCache
//!
//! A GPU texture cache for image files, keyed by path.  An image is decoded with `SDL2_image`
//! once, the first time any `Widget` asks for it; every later draw copies the cached texture.
//! A file that fails to load is not cached, so it is tried again once it exists.

use pushrod::error::Error;
use pushrod::geometry::Size;
use sdl2::image::LoadTexture;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::collections::HashMap;

/// One cached image: its texture and pixel size.
struct Entry {
    texture: Texture,
    size: Size,
}

/// Image textures shared by every `Widget`, keyed by file path.
#[derive(Default)]
pub struct ImageCache {
    entries: HashMap<String, Entry>,
}

impl std::fmt::Debug for ImageCache {
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
        f.debug_struct("ImageCache")
            .field("images", &self.entries.len())
            .finish()
    }
}

impl ImageCache {
    /// Creates an empty cache.
    ///
    /// # Returns
    ///
    /// The new `ImageCache`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns how many images are cached.
    ///
    /// # Returns
    ///
    /// The number of cached textures.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the cache holds no images.
    ///
    /// # Returns
    ///
    /// `true` when empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Loads an image into the cache unless it is already there.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The canvas whose renderer owns the texture.
    /// * `path` - Path to the image file.
    ///
    /// # Returns
    ///
    /// The image's size in pixels.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Image`] if the file cannot be read or decoded.
    pub fn load(&mut self, canvas: &Canvas<Window>, path: &str) -> Result<Size, Error> {
        if let Some(entry) = self.entries.get(path) {
            return Ok(entry.size);
        }

        let texture = canvas
            .texture_creator()
            .load_texture(path)
            .map_err(|message| Error::image(path, message))?;
        let query = texture.query();
        let size = Size::new(query.width, query.height);

        self.entries
            .insert(path.to_string(), Entry { texture, size });

        Ok(size)
    }

    /// Returns a cached texture.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file, as passed to [`load`](ImageCache::load).
    ///
    /// # Returns
    ///
    /// The texture, or `None` if the image has not been loaded.
    pub fn texture(&self, path: &str) -> Option<&Texture> {
        self.entries.get(path).map(|entry| &entry.texture)
    }
}

impl Drop for ImageCache {
    /// Frees every cached texture (`unsafe_textures` textures are not freed on drop).
    fn drop(&mut self) {
        for (_, entry) in self.entries.drain() {
            // SAFETY: the backend declares the cache before its canvas, so the renderer that
            // owns these textures is still alive here.
            unsafe { entry.texture.destroy() };
        }
    }
}
