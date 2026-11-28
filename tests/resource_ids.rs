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

//! Tests for the typed resource ID newtypes.

use pushrod::form::FormIndex as FormFormIndex;
use pushrod::resource::{AlertId, FileId, FormId, FormIndex, IconId, ImageId, MenuId, StringId};
use std::collections::HashSet;

#[test]
fn ids_convert_to_their_number() {
    assert_eq!(StringId(3).get(), 3);
    assert_eq!(u32::from(FormId(1)), 1);
    assert_eq!(u32::from(ImageId(2)), 2);
    assert_eq!(u32::from(IconId(4)), 4);
    assert_eq!(u32::from(FileId(5)), 5);
    assert_eq!(u32::from(MenuId(6)), 6);
    assert_eq!(u32::from(AlertId(7)), 7);
}

#[test]
fn ids_of_one_kind_compare_hash_and_order() {
    let ids: HashSet<StringId> = [StringId(1), StringId(1), StringId(2)].into();

    assert_eq!(ids.len(), 2);
    assert!(StringId(1) < StringId(2));
    assert_eq!(ImageId(9), ImageId(9));
}

#[test]
fn form_index_is_the_form_modules_type() {
    let index: FormFormIndex = FormIndex(7);

    assert_eq!(u32::from(index), 7);
}
