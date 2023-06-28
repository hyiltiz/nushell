use nu_engine::scope::ScopeData;
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, IntoInterruptiblePipelineData, PipelineData, ShellError, Signature, Type,
};

#[derive(Clone)]
pub struct ScopeCommands;

impl Command for ScopeCommands {
    fn name(&self) -> &str {
        "scope commands"
    }

    fn signature(&self) -> Signature {
        Signature::build("scope commands")
            .input_output_types(vec![(Type::Nothing, Type::Any)])
            .allow_variants_without_examples(true)
            .category(Category::Filters)
    }

    fn usage(&self) -> &str {
        "Output info on the commands in the current scope."
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let span = call.head;
        let ctrlc = engine_state.ctrlc.clone();

        let mut scope_data = ScopeData::new(engine_state, stack);
        scope_data.populate_all();

        Ok(scope_data.collect_commands(span).into_pipeline_data(ctrlc))
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Show the commands in the current scope",
            example: "scope commands",
            result: None,
        }]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(ScopeCommands {})
    }
}