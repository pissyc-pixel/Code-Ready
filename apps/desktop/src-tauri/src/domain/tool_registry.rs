use super::contracts::{
    PlatformId, PlatformPolicy, ToolCapability, ToolDefinition, ToolId, ToolRequirement,
};

pub fn built_in_tool_registry() -> Vec<ToolDefinition> {
    vec![
        tool(
            ToolId::Winget,
            "tools.winget.name",
            ToolRequirement::Default,
            ToolRequirement::Unavailable,
        ),
        tool(
            ToolId::Git,
            "tools.git.name",
            ToolRequirement::Default,
            ToolRequirement::Default,
        ),
        tool(
            ToolId::Nodejs,
            "tools.nodejsAndNpm.name",
            ToolRequirement::Default,
            ToolRequirement::Optional,
        ),
        tool(
            ToolId::ClaudeCode,
            "tools.claudeCode.name",
            ToolRequirement::Default,
            ToolRequirement::Default,
        ),
        tool(
            ToolId::CodexCli,
            "tools.codexCli.name",
            ToolRequirement::Default,
            ToolRequirement::Default,
        ),
    ]
}

fn tool(
    id: ToolId,
    label_key: &str,
    windows_requirement: ToolRequirement,
    macos_requirement: ToolRequirement,
) -> ToolDefinition {
    ToolDefinition {
        id,
        label_key: label_key.to_owned(),
        platform_policies: vec![
            PlatformPolicy {
                platform: PlatformId::WindowsX64,
                requirement: windows_requirement,
            },
            PlatformPolicy {
                platform: PlatformId::MacosArm64,
                requirement: macos_requirement,
            },
        ],
        capabilities: vec![
            ToolCapability::Detect,
            ToolCapability::Install,
            ToolCapability::Upgrade,
            ToolCapability::Repair,
        ],
        runtime_dependencies: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> Vec<ToolDefinition> {
        built_in_tool_registry()
    }

    #[test]
    fn registry_tool_ids_are_unique() {
        let tools = registry();

        for (index, tool) in tools.iter().enumerate() {
            assert!(!tools[..index].iter().any(|previous| previous.id == tool.id));
        }
    }

    #[test]
    fn every_tool_has_exactly_one_policy_per_supported_platform() {
        for tool in registry() {
            assert_eq!(
                tool.platform_policies
                    .iter()
                    .filter(|policy| policy.platform == PlatformId::WindowsX64)
                    .count(),
                1
            );
            assert_eq!(
                tool.platform_policies
                    .iter()
                    .filter(|policy| policy.platform == PlatformId::MacosArm64)
                    .count(),
                1
            );
        }
    }

    #[test]
    fn every_tool_has_unique_capabilities() {
        for tool in registry() {
            for (index, capability) in tool.capabilities.iter().enumerate() {
                assert!(!tool.capabilities[..index].contains(capability));
            }
        }
    }

    #[test]
    fn runtime_dependencies_reference_known_tools_and_not_themselves() {
        let tools = registry();

        for tool in &tools {
            for dependency in &tool.runtime_dependencies {
                assert_ne!(&tool.id, dependency);
                assert!(tools.iter().any(|candidate| &candidate.id == dependency));
            }
        }
    }

    #[test]
    fn runtime_dependency_graph_is_acyclic() {
        fn visit(
            index: usize,
            tools: &[ToolDefinition],
            visiting: &mut [bool],
            visited: &mut [bool],
        ) {
            if visited[index] {
                return;
            }
            assert!(
                !visiting[index],
                "dependency cycle at {:?}",
                tools[index].id
            );
            visiting[index] = true;

            for dependency in &tools[index].runtime_dependencies {
                let dependency_index = tools
                    .iter()
                    .position(|candidate| &candidate.id == dependency)
                    .expect("dependencies are checked separately");
                visit(dependency_index, tools, visiting, visited);
            }

            visiting[index] = false;
            visited[index] = true;
        }

        let tools = registry();
        let mut visiting = vec![false; tools.len()];
        let mut visited = vec![false; tools.len()];

        for index in 0..tools.len() {
            visit(index, &tools, &mut visiting, &mut visited);
        }
    }

    fn assert_tool(
        tool: &ToolDefinition,
        id: ToolId,
        label_key: &str,
        windows_requirement: ToolRequirement,
        macos_requirement: ToolRequirement,
    ) {
        assert_eq!(&tool.id, &id);
        assert_eq!(tool.label_key, label_key);
        assert_eq!(
            &tool.platform_policies,
            &vec![
                PlatformPolicy {
                    platform: PlatformId::WindowsX64,
                    requirement: windows_requirement,
                },
                PlatformPolicy {
                    platform: PlatformId::MacosArm64,
                    requirement: macos_requirement,
                },
            ]
        );
        assert_eq!(
            &tool.capabilities,
            &vec![
                ToolCapability::Detect,
                ToolCapability::Install,
                ToolCapability::Upgrade,
                ToolCapability::Repair,
            ]
        );
        assert!(tool.runtime_dependencies.is_empty());
    }

    #[test]
    fn registry_matches_the_fixed_slice_zero_matrix() {
        let tools = registry();
        assert_eq!(tools.len(), 5);

        assert_tool(
            &tools[0],
            ToolId::Winget,
            "tools.winget.name",
            ToolRequirement::Default,
            ToolRequirement::Unavailable,
        );
        assert_tool(
            &tools[1],
            ToolId::Git,
            "tools.git.name",
            ToolRequirement::Default,
            ToolRequirement::Default,
        );
        assert_tool(
            &tools[2],
            ToolId::Nodejs,
            "tools.nodejsAndNpm.name",
            ToolRequirement::Default,
            ToolRequirement::Optional,
        );
        assert_tool(
            &tools[3],
            ToolId::ClaudeCode,
            "tools.claudeCode.name",
            ToolRequirement::Default,
            ToolRequirement::Default,
        );
        assert_tool(
            &tools[4],
            ToolId::CodexCli,
            "tools.codexCli.name",
            ToolRequirement::Default,
            ToolRequirement::Default,
        );
    }

    #[test]
    fn claude_and_codex_have_no_runtime_dependencies() {
        let tools = registry();

        assert!(
            tools
                .iter()
                .find(|tool| tool.id == ToolId::ClaudeCode)
                .expect("Claude Code is registered")
                .runtime_dependencies
                .is_empty()
        );
        assert!(
            tools
                .iter()
                .find(|tool| tool.id == ToolId::CodexCli)
                .expect("Codex CLI is registered")
                .runtime_dependencies
                .is_empty()
        );
    }

    #[test]
    fn nodejs_definition_names_npm_as_a_derived_capability_without_an_npm_tool() {
        let tools = registry();
        let nodejs = tools
            .iter()
            .find(|tool| tool.id == ToolId::Nodejs)
            .expect("Node.js is registered");

        assert_eq!(nodejs.label_key, "tools.nodejsAndNpm.name");
        assert_eq!(tools.len(), 5);
        assert!(tools.iter().all(|tool| tool.label_key != "tools.npm.name"));
    }
}
