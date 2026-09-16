//! Verb trees project validated context ownership and signatures without interpreting mock output.

use crate::cli_definition::{CliBehaviorBinding, CliContextParent, CliDefinition};
use std::fmt::Write;

/// Render the full CLI verb tree from a validated definition, with sorted commands before child contexts.
pub(crate) fn render_cli_verb_tree(definition: &CliDefinition) -> String {
    let mut output = format!("{}\n", definition.id);
    append_cli_context_tree(definition, None, "root", "", true, &mut output);
    output
}

/// Direct trees use the route projection's local words and omit authoring control syntax.
pub(crate) fn render_run_verb_tree(
    definition: &CliDefinition,
    routes: &crate::cli_run_routes::CliRunRoutes,
    detailed: bool,
) -> String {
    let mut output = format!("{}\n", definition.id);
    append_cli_context_tree(
        definition,
        Some(routes),
        "root",
        "",
        detailed || routes.operator.is_none(),
        &mut output,
    );
    output
}

pub(crate) fn cli_binding_label(binding: &CliBehaviorBinding) -> &'static str {
    match binding {
        CliBehaviorBinding::Simulated { .. } => "simulated",
        CliBehaviorBinding::Unbound { .. } => "unbound",
        CliBehaviorBinding::Connected {
            provider: crate::cli_definition::CliConnectedProvider::JsonFileRead,
            ..
        } => "connected:json-file-read-v1",
        CliBehaviorBinding::Connected {
            provider: crate::cli_definition::CliConnectedProvider::JsonHttpGet,
            ..
        } => "connected:json-http-get-v1",
    }
}

fn append_cli_context_tree(
    definition: &CliDefinition,
    routes: Option<&crate::cli_run_routes::CliRunRoutes>,
    context_id: &str,
    prefix: &str,
    detailed: bool,
    output: &mut String,
) {
    let context = &definition.contexts[context_id];
    let children: Vec<_> = if let Some(routes) = routes {
        routes
            .context_children(context_id)
            .iter()
            .map(|(word, id)| (word.as_str(), id.as_str()))
            .collect()
    } else {
        definition
        .contexts
        .iter()
        .filter(|(_, child)| {
            matches!(&child.parent, CliContextParent::Context(parent) if parent == context_id)
        })
        .map(|(id, _)| (id.as_str(), id.as_str())).collect()
    };
    let commands: Vec<_> = context
        .commands
        .iter()
        .filter(|(_, command)| {
            detailed
                || context_id != "root"
                || !routes
                    .and_then(|routes| routes.operator.as_ref())
                    .is_some_and(|profile| profile.command_aliases.contains_key(&command.id))
        })
        .collect();
    let endpoint_control = routes
        .filter(|routes| routes.operator.is_some() && context_id == "root")
        .map(|routes| routes.preferred_run_control(":endpoint"));
    let count = commands.len() + children.len() + usize::from(endpoint_control.is_some());
    for (index, (word, command)) in commands.iter().enumerate() {
        output.push_str(prefix);
        output.push_str(if index + 1 == count { "`-- " } else { "|-- " });
        if routes.is_none() {
            output.push_str("invoke ");
        }
        output.push_str(word);
        for parameter in &command.parameters {
            write!(output, " <{}:{}>", parameter.name, parameter.value_type).unwrap();
        }
        let binding = cli_binding_label(&command.binding);
        if detailed || binding == "simulated" || binding == "unbound" {
            writeln!(output, " [{binding}]").unwrap();
        } else {
            output.push('\n');
        }
    }
    for (index, (word, id)) in children.iter().enumerate() {
        let last = commands.len() + index + 1 == count;
        output.push_str(prefix);
        output.push_str(if last { "`-- " } else { "|-- " });
        writeln!(output, "{word}/").unwrap();
        let continuation = if last { "    " } else { "|   " };
        append_cli_context_tree(
            definition,
            routes,
            id,
            &format!("{prefix}{continuation}"),
            detailed,
            output,
        );
    }
    if let Some(control) = endpoint_control {
        writeln!(output, "{prefix}`-- {control}").unwrap();
        for (index, (action, _)) in crate::cli_operator_settings::CLI_ENDPOINT_ACTIONS
            .iter()
            .enumerate()
        {
            let branch = if index + 1 == crate::cli_operator_settings::CLI_ENDPOINT_ACTIONS.len() {
                "`--"
            } else {
                "|--"
            };
            let arguments = match *action {
                "set" => " <url> [--save]",
                "clear" => " [--save]",
                _ => "",
            };
            writeln!(output, "{prefix}    {branch} {action}{arguments}").unwrap();
        }
    }
}
