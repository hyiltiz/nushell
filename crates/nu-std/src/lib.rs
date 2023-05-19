use std::path::PathBuf;

use nu_engine::{env::current_dir, eval_block};
use nu_parser::parse;
use nu_protocol::engine::{Stack, StateWorkingSet, VirtualPath};
use nu_protocol::{report_error, PipelineData};

// Virtual std directory unlikely to appear in user's file system
const NU_STDLIB_VIRTUAL_DIR: &str = "NU_STDLIB_VIRTUAL_DIR";

pub fn load_standard_library(
    engine_state: &mut nu_protocol::engine::EngineState,
) -> Result<(), miette::ErrReport> {
    let (block, delta) = {
        // Using full virtual path to avoid potential conflicts with user having 'std' directory
        // in their working directory.
        let std_dir = PathBuf::from(NU_STDLIB_VIRTUAL_DIR).join("std");

        // these modules are loaded in the order they appear in this list
        #[rustfmt::skip]
        let submodules = vec![
            // helper modules that could be used in other parts of the library
            ("log", include_str!("../lib/log.nu")),

            // the rest of the library
            ("mod.nu", include_str!("../std/mod.nu")),
            ("assert.nu", include_str!("../std/assert.nu")),
            ("input.nu", include_str!("../std/input.nu")),
            ("dirs", include_str!("../std/dirs.nu")), // moved from lib to std
            // FIXME: the files above are in ../std/*.nu
            //        the ones below are in ../lib/*.nu
            //        Is this expected?
            ("iter", include_str!("../lib/iter.nu")),
            ("help", include_str!("../lib/help.nu")),
            ("testing", include_str!("../lib/testing.nu")),
            ("xml", include_str!("../lib/xml.nu")),
            ("dt", include_str!("../lib/dt.nu")),
        ];

        // Define commands to be preloaded into the default (top level, unprefixed) namespace.
        // User can invoke these without having to `use std` beforehand.
        // Entries are: (name to add to default namespace, path under std to find implementation)
        //
        // Conventionally, for a command implemented as `std foo`, the name added
        // is either `std foo` or bare `foo`, not some arbitrary rename.

        #[rustfmt::skip]
        let prelude = vec![
            ("std help", "help"),
            ("std help commands", "help commands"),
            ("std help aliases", "help aliases"),
            ("std help modules", "help modules"),
            ("std help externs", "help externs"),
            ("std help operators", "help operators"),

            ("enter", "enter"),
            ("shells", "shells"),
            ("g", "g"),
            ("n", "n"),
            ("p", "p"),
            ("dexit", "dexit"),
            ("cdhook", "cdhook"),
        ];

        let mut working_set = StateWorkingSet::new(engine_state);
        let mut std_virt_paths = vec![];

        for (name, content) in std_files.drain(..) {
            let name = std_dir.join(name);

            let file_id =
                working_set.add_file(name.to_string_lossy().to_string(), content.as_bytes());
            let virtual_file_id = working_set.add_virtual_path(
                name.to_string_lossy().to_string(),
                VirtualPath::File(file_id),
            );
            std_virt_paths.push(virtual_file_id);
        }

        let std_dir = std_dir.to_string_lossy().to_string();
        let source = format!(
            r#"
# Define the `std` module
module {std_dir}

# Prelude
use std dirs [
    enter
    shells
    g
    n
    p
    dexit
]
use std pwd
"#
        );

        let _ = working_set.add_virtual_path(std_dir, VirtualPath::Dir(std_virt_paths));

        // Change the currently parsed directory
        let prev_currently_parsed_cwd = working_set.currently_parsed_cwd.clone();
        working_set.currently_parsed_cwd = Some(PathBuf::from(NU_STDLIB_VIRTUAL_DIR));

        let block = parse(
            &mut working_set,
            Some("loading stdlib"),
            source.as_bytes(),
            false,
        );

        if let Some(err) = working_set.parse_errors.first() {
            report_error(&working_set, err);
        }

        // Restore the currently parsed directory back
        working_set.currently_parsed_cwd = prev_currently_parsed_cwd;

        (block, working_set.render())
    };

    engine_state.merge_delta(delta)?;

    // We need to evaluate the module in order to run the `export-env` blocks.
    let mut stack = Stack::new();
    let pipeline_data = PipelineData::Empty;
    eval_block(
        engine_state,
        &mut stack,
        &block,
        pipeline_data,
        false,
        false,
    )?;

    let cwd = current_dir(engine_state, &stack)?;
    engine_state.merge_env(&mut stack, cwd)?;

    Ok(())
}
