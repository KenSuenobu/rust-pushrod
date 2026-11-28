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

//! `pushrod-form-ids <form-file> <out.rs>`: writes the Rust ID constants of a form file (see
//! `pushrod::form::codegen`).  Exits with status 0 on success and 1 with a message on failure.

use std::process::ExitCode;

/// Runs the command.
///
/// # Returns
///
/// Success, or failure after printing the reason to standard error.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() != 2 {
        eprintln!("usage: pushrod-form-ids <form-file> <out.rs>");
        return ExitCode::FAILURE;
    }
    match pushrod::form::codegen::write_ids(&args[0], &args[1]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("pushrod-form-ids: {error}");
            ExitCode::FAILURE
        }
    }
}
