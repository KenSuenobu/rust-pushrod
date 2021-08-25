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

/// Provides a definition of a point of origin: contains the `x` and `y` coordinates of
/// an object.
#[derive(Default, Copy, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Provides a definition of the size of an object: contains the `w` and `h` coordinates of
/// an object.
#[derive(Default, Copy, Clone)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            w: width,
            h: height,
        }
    }

    pub fn get_memory_size(&self) -> u32 {
        self.w * self.h * 4
    }
}