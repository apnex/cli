//! Compact operator help projects complete discovery metadata without changing the command surface.

use crate::cli_run_frontend::escape_run_metadata;
use crate::cli_run_session::CliRunSession;
use serde_json::Value;
use std::fmt::Write;

pub(crate) fn operator_alias_target(
    session: &CliRunSession,
    command_id: &str,
    current: &str,
) -> Option<String> {
    let target = session
        .routes
        .operator
        .as_ref()?
        .command_aliases
        .get(command_id)?;
    for (context, definition) in &session.active_interface().definition.contexts {
        if let Some((word, _)) = definition
            .commands
            .iter()
            .find(|(_, command)| &command.id == target)
        {
            let path = session
                .routes
                .context_path(&session.active_interface().definition, context);
            return Some(if context == current || path.is_empty() {
                word.clone()
            } else {
                format!("{path} {word}")
            });
        }
    }
    None
}

pub(crate) fn render_operator_help(
    session: &CliRunSession,
    context: &str,
    command_word: Option<&str>,
    commands: &[Value],
    children: &[Value],
) -> String {
    let active = session.active_interface();
    let profile = session.routes.operator.as_ref().unwrap();
    let path = session.routes.context_path(&active.definition, context);
    let mut output = format!(
        "{}{}\n",
        active.definition.id,
        if path.is_empty() {
            String::new()
        } else {
            format!(" / {path}")
        }
    );
    let endpoint_control = session.routes.preferred_run_control(":endpoint");
    if let Some(settings) = &session.endpoint_settings {
        let endpoint = settings.endpoint.as_deref().unwrap_or("not configured");
        writeln!(output, "Endpoint: {endpoint}").unwrap();
        if settings.endpoint.is_none() {
            writeln!(
                output,
                "Use {endpoint_control} set <url> to select an endpoint."
            )
            .unwrap();
        }
    }
    if let Some(word) = command_word {
        let command = &active.definition.contexts[context].commands[word];
        write!(
            output,
            "\nUsage: {}{} {word}",
            active.definition.id,
            if path.is_empty() {
                String::new()
            } else {
                format!(" {path}")
            }
        )
        .unwrap();
        for parameter in &command.parameters {
            write!(output, " <{}>", parameter.name).unwrap();
        }
        writeln!(output, "\n{}", escape_run_metadata(&command.help)).unwrap();
        if let Some(target) = operator_alias_target(session, &command.id, context) {
            writeln!(output, "Alias for: {target}").unwrap();
        }
        for parameter in &command.parameters {
            writeln!(
                output,
                "  {}: {}",
                parameter.name,
                escape_run_metadata(&parameter.help)
            )
            .unwrap();
        }
        output
            .push_str("Output: --table, --json, --view <name>; --events for structured details.\n");
        return output;
    }
    if !children.is_empty() {
        output.push_str("\nContexts\n");
        let width = children
            .iter()
            .map(|child| child["word"].as_str().unwrap().len())
            .max()
            .unwrap_or(0);
        for child in children {
            writeln!(
                output,
                "  {:width$}  {}",
                child["word"].as_str().unwrap(),
                escape_run_metadata(child["help"].as_str().unwrap())
            )
            .unwrap();
        }
    }
    let visible: Vec<_> = commands
        .iter()
        .filter(|command| {
            context != "root"
                || !profile
                    .command_aliases
                    .contains_key(command["id"].as_str().unwrap())
        })
        .collect();
    if !visible.is_empty() {
        output.push_str("\nCommands\n");
        let width = visible
            .iter()
            .map(|command| command["word"].as_str().unwrap().len())
            .max()
            .unwrap_or(0);
        for command in visible {
            let help = operator_alias_target(session, command["id"].as_str().unwrap(), context)
                .map(|target| format!("Alias for {target}"))
                .unwrap_or_else(|| escape_run_metadata(command["help"].as_str().unwrap()));
            let label = match command["binding"].as_str() {
                Some("simulated") => " [simulated]",
                Some("unbound") => " [unbound]",
                _ => "",
            };
            writeln!(
                output,
                "  {:width$}  {help}{label}",
                command["word"].as_str().unwrap()
            )
            .unwrap();
        }
    }
    let listing: Vec<_> = profile
        .context_listing
        .iter()
        .filter(|word| {
            !active.definition.contexts[context]
                .commands
                .contains_key(*word)
                && !session.routes.context_children(context).contains_key(*word)
        })
        .cloned()
        .collect();
    if !listing.is_empty() {
        writeln!(output, "\n{}: list this context", listing.join(", ")).unwrap();
    }
    if session.endpoint_settings.is_some() {
        writeln!(
            output,
            "Management: {endpoint_control} show | set <url> | clear | save | load"
        )
        .unwrap();
    }
    writeln!(
        output,
        "Navigation: <context>, {}, {}{}",
        session.routes.preferred_run_control(":up"),
        session.routes.preferred_run_control(":top"),
        if session
            .routes
            .control_aliases
            .get("/")
            .is_some_and(|target| target == ":top")
        {
            " (/)"
        } else {
            ""
        }
    )
    .unwrap();
    let help = session.routes.preferred_run_control(":help");
    writeln!(
        output,
        "Help: {}{help} <command>; {help} --all for full details",
        if session
            .routes
            .control_aliases
            .get("?")
            .is_some_and(|target| target == ":help")
        {
            "? or "
        } else {
            ""
        }
    )
    .unwrap();
    writeln!(
        output,
        "Structure: {}    Leave: {}",
        session.routes.preferred_run_control(":tree"),
        session.routes.preferred_run_control(":exit")
    )
    .unwrap();
    output
}

pub(crate) fn render_operator_context_hint(session: &CliRunSession) -> String {
    let active = session.active_interface();
    let context = &active.definition.contexts[&active.context];
    let words: Vec<_> = context
        .commands
        .keys()
        .filter(|word| {
            active.context != "root"
                || !session
                    .routes
                    .operator
                    .as_ref()
                    .unwrap()
                    .command_aliases
                    .contains_key(&context.commands[*word].id)
        })
        .cloned()
        .collect();
    let help = session.routes.preferred_run_control(":help");
    if words.is_empty() {
        format!("Use {help} to list contexts and commands.\n")
    } else {
        format!("Commands: {}. Use {help} for details.\n", words.join(", "))
    }
}
