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

/// Geometry definitions: provides a point and size.
pub enum Geometry {
    Point { x: i32, y: i32 },
    Size { w: u32, h: u32 },
}

impl Geometry {
    /// Returns the amount of memory that a texture would use based on the geometry width and
    /// height of the geometry, as long as `Self::Size` is used.  If not, a 0 will be sent,
    /// as this is dependent upon the `Size` enum being used.
    fn get_memory_size(&self) -> u32 {
        return match self {
            Self::Size { w, h } => w * h * 4,
            _default => 0,
        }
    }
}
