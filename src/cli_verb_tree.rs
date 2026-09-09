//! Verb trees project validated context ownership and signatures without interpreting mock output.

use crate::cli_definition::{CliBehaviorBinding, CliContextParent, CliDefinition};
use std::fmt::Write;

/// Render the full CLI verb tree from a validated definition, with sorted commands before child contexts.
pub(crate) fn render_cli_verb_tree(definition: &CliDefinition) -> String {
    let mut output = format!("{}\n", definition.id);
    append_cli_context_tree(definition, None, "root", "", &mut output);
    output
}

/// Direct trees use the route projection's local words and omit authoring control syntax.
pub(crate) fn render_run_verb_tree(
    definition: &CliDefinition,
    routes: &crate::cli_run_routes::CliRunRoutes,
) -> String {
    let mut output = format!("{}\n", definition.id);
    append_cli_context_tree(definition, Some(routes), "root", "", &mut output);
    output
}

pub(crate) fn cli_binding_label(binding: &CliBehaviorBinding) -> &'static str {
    match binding {
        CliBehaviorBinding::Simulated { .. } => "simulated",
        CliBehaviorBinding::Unbound { .. } => "unbound",
        CliBehaviorBinding::Connected { .. } => "connected:json-file-read-v1",
    }
}

fn append_cli_context_tree(
    definition: &CliDefinition,
    routes: Option<&crate::cli_run_routes::CliRunRoutes>,
    context_id: &str,
    prefix: &str,
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
    let count = context.commands.len() + children.len();
    for (index, (word, command)) in context.commands.iter().enumerate() {
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
        writeln!(output, " [{binding}]").unwrap();
    }
    for (index, (word, id)) in children.iter().enumerate() {
        let last = context.commands.len() + index + 1 == count;
        output.push_str(prefix);
        output.push_str(if last { "`-- " } else { "|-- " });
        writeln!(output, "{word}/").unwrap();
        let continuation = if last { "    " } else { "|   " };
        append_cli_context_tree(
            definition,
            routes,
            id,
            &format!("{prefix}{continuation}"),
            output,
        );
    }
}
