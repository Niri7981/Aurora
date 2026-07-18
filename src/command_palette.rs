#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlashCommand {
    pub name: &'static str,
    pub description: &'static str,
}

const SLASH_COMMANDS: [SlashCommand; 9] = [
    SlashCommand {
        name: "/clear",
        description: "Clear the current session",
    },
    SlashCommand {
        name: "/context",
        description: "Preview local context",
    },
    SlashCommand {
        name: "/context init",
        description: "Create local context files",
    },
    SlashCommand {
        name: "/context preview",
        description: "Preview provider-filtered context",
    },
    SlashCommand {
        name: "/model",
        description: "Browse and switch models",
    },
    SlashCommand {
        name: "/mcp log",
        description: "Inspect recent MCP context access",
    },
    SlashCommand {
        name: "/resume",
        description: "Resume a saved session",
    },
    SlashCommand {
        name: "/tools",
        description: "Inspect the active tool catalog",
    },
    SlashCommand {
        name: "/tools log",
        description: "Inspect recent tool results",
    },
];

pub fn slash_commands() -> &'static [SlashCommand] {
    &SLASH_COMMANDS
}

pub fn matching_commands(input: &str) -> Vec<&'static SlashCommand> {
    if !input.starts_with('/') {
        return Vec::new();
    }

    slash_commands()
        .iter()
        .filter(|command| command.name.starts_with(input))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::matching_commands;

    #[test]
    fn slash_lists_all_commands() {
        assert_eq!(matching_commands("/").len(), 9);
    }

    #[test]
    fn partial_command_filters_to_resume() {
        let matches = matching_commands("/re");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "/resume");
    }

    #[test]
    fn ordinary_text_has_no_command_suggestions() {
        assert!(matching_commands("打开 Safari").is_empty());
    }
}
