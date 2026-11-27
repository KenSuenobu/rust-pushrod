// Property Introspection Example
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

//! Lists a widget's properties, changes one by name, and shows the `PropertyChanged` event.  Run
//! it with `cargo run -p pushrod-widgets --example properties`.

use pushrod::event::EngineEventKind;
use pushrod::prelude::*;
use pushrod::registry::PropertyValue;
use pushrod_widgets::prelude::*;
use std::time::Instant;

fn main() {
    let mut engine: Engine<()> = Engine::new(size(300, 100), 30);
    let id = engine.add_widget(Slider::new(0.0, 100.0));

    engine.on_engine_event_filtered(&[EngineEventKind::PropertyChanged], |_s, _ui, event| {
        println!("event: {event:?}");
    });

    println!("Properties of widget {id}:");

    for property in engine.widgets().get(id).unwrap().properties() {
        println!(
            "  {} ({:?}) = {:?}",
            property.name, property.kind, property.value
        );
    }

    // Change the value and a core property by name.
    for (name, value) in [
        ("value", PropertyValue::Float(40.0)),
        ("value", PropertyValue::Float(40.0)),
        ("width", PropertyValue::Int(150)),
        ("vertical", PropertyValue::Bool(true)),
        ("nope", PropertyValue::Int(1)),
    ] {
        match engine.set_widget_property(id, name, &value) {
            Ok(changed) => println!("set {name} = {value:?}: changed = {changed}"),
            Err(error) => println!("set {name} = {value:?}: {error}"),
        }
    }

    // The events are delivered with the next round of event handling.
    engine.advance(Instant::now(), &mut ());
}
