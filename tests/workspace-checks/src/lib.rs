//! Executable assertions for workspace and CI invariants.

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output};

    const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

    #[derive(Debug, Eq, PartialEq)]
    struct TreeNode<'a> {
        depth: usize,
        name: &'a str,
        version: &'a str,
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("the workspace root must exist")
    }

    fn cargo_tree(arguments: &[&str]) -> Result<String, String> {
        let output = Command::new(env!("CARGO"))
            .arg("tree")
            .args(arguments)
            .args(["--locked", "--prefix", "depth", "--format", "{p}"])
            .current_dir(workspace_root())
            .output()
            .map_err(|error| format!("failed to run cargo tree: {error}"))?;

        command_stdout("cargo tree", output)
    }

    fn command_stdout(command: &str, output: Output) -> Result<String, String> {
        if !output.status.success() {
            return Err(format!(
                "{command} failed with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        String::from_utf8(output.stdout)
            .map_err(|error| format!("{command} produced non-UTF-8 output: {error}"))
    }

    fn parse_tree(output: &str) -> Result<Vec<TreeNode<'_>>, String> {
        let mut nodes: Vec<TreeNode<'_>> = Vec::new();

        for (index, line) in output.lines().enumerate() {
            let line_number = index + 1;
            if line.is_empty() {
                return Err(format!("cargo tree line {line_number} is empty"));
            }

            let depth_end = line
                .find(|character: char| !character.is_ascii_digit())
                .ok_or_else(|| format!("cargo tree line {line_number} contains only a depth"))?;
            if depth_end == 0 {
                return Err(format!(
                    "cargo tree line {line_number} has no structural depth prefix: {line:?}"
                ));
            }

            let depth = line[..depth_end].parse::<usize>().map_err(|error| {
                format!("cargo tree line {line_number} has an invalid depth: {error}")
            })?;
            let mut package = line[depth_end..].split_whitespace();
            let name = package
                .next()
                .ok_or_else(|| format!("cargo tree line {line_number} has no package name"))?;
            let version = package
                .next()
                .and_then(|value| value.strip_prefix('v'))
                .ok_or_else(|| {
                    format!("cargo tree line {line_number} has no package version: {line:?}")
                })?;

            if nodes.is_empty() && depth != 0 {
                return Err(format!(
                    "cargo tree root has depth {depth}; expected depth 0"
                ));
            }
            if let Some(previous) = nodes.last()
                && depth > previous.depth + 1
            {
                return Err(format!(
                    "cargo tree depth jumps from {} to {depth} on line {line_number}",
                    previous.depth
                ));
            }

            nodes.push(TreeNode {
                depth,
                name,
                version,
            });
        }

        if nodes.is_empty() {
            return Err("cargo tree returned no packages".to_owned());
        }

        Ok(nodes)
    }

    fn check_facade_tree(output: &str) -> Result<(), String> {
        let nodes = parse_tree(output)?;
        let expected_root = TreeNode {
            depth: 0,
            name: "libneo",
            version: PACKAGE_VERSION,
        };

        if nodes.first() != Some(&expected_root) {
            return Err(format!(
                "the facade dependency tree has the wrong root: expected {expected_root:?}, got {:?}",
                nodes.first()
            ));
        }
        if nodes.len() != 1 {
            return Err(format!(
                "the default libneo facade must have no dependencies; cargo tree contained {} package nodes: {nodes:?}",
                nodes.len()
            ));
        }

        Ok(())
    }

    fn check_gpui_tree(output: &str) -> Result<(), String> {
        let nodes = parse_tree(output)?;
        let adapter = nodes.iter().find(|node| {
            node.name == "libneo-gpui" && node.version == PACKAGE_VERSION && node.depth == 1
        });

        if adapter.is_none() {
            return Err(format!(
                "the libneo gpui feature must resolve directly to libneo-gpui v{PACKAGE_VERSION}; parsed nodes: {nodes:?}"
            ));
        }

        Ok(())
    }

    fn check_sdk_with(program: &Path) -> Result<(), String> {
        let output = Command::new(program)
            .arg("--show-sdk-version")
            .output()
            .map_err(|error| format!("failed to run {}: {error}", program.display()))?;
        let stdout = command_stdout(&program.display().to_string(), output)?;
        check_sdk_version(&stdout)
    }

    fn check_sdk_version(output: &str) -> Result<(), String> {
        let version = output.trim();
        let major = version
            .split_once('.')
            .map_or(version, |(major, _)| major)
            .parse::<u32>()
            .map_err(|error| {
                format!("xcrun returned an unparseable SDK version {version:?}: {error}")
            })?;

        if major < 26 {
            return Err(format!(
                "the macOS SDK must be version 26 or later; xcrun reported {version:?}"
            ));
        }

        Ok(())
    }

    #[test]
    fn facade_has_no_dependencies() {
        let tree = cargo_tree(&["-p", "libneo", "--no-default-features"])
            .unwrap_or_else(|error| panic!("facade dependency check failed: {error}"));
        println!("{tree}");
        check_facade_tree(&tree)
            .unwrap_or_else(|error| panic!("facade dependency check failed: {error}"));
    }

    #[test]
    fn gpui_feature_includes_adapter() {
        let tree = cargo_tree(&["-p", "libneo", "--features", "gpui"])
            .unwrap_or_else(|error| panic!("gpui feature check failed: {error}"));
        println!("{tree}");
        check_gpui_tree(&tree).unwrap_or_else(|error| panic!("gpui feature check failed: {error}"));
    }

    #[test]
    fn installed_sdk_is_supported() {
        check_sdk_with(Path::new("xcrun"))
            .unwrap_or_else(|error| panic!("macOS SDK check failed: {error}"));
    }

    #[test]
    fn facade_check_rejects_a_dependency() {
        let tree = format!("0libneo v{PACKAGE_VERSION}\n1unexpected v1.0.0\n");
        let error = check_facade_tree(&tree).expect_err("a facade dependency must fail the check");
        assert!(error.contains("must have no dependencies"), "{error}");
    }

    #[test]
    fn gpui_check_rejects_a_missing_adapter() {
        let tree = format!("0libneo v{PACKAGE_VERSION}\n1gpui v0.2.2\n");
        let error = check_gpui_tree(&tree).expect_err("a missing adapter must fail the check");
        assert!(error.contains("must resolve directly"), "{error}");
    }

    #[test]
    fn sdk_check_rejects_a_missing_xcrun() {
        let error = check_sdk_with(Path::new("/definitely-not-an-installed-program"))
            .expect_err("a missing xcrun must fail the check");
        assert!(error.contains("failed to run"), "{error}");
    }

    #[test]
    fn sdk_check_rejects_unparseable_output() {
        let error = check_sdk_version("not-a-version\n")
            .expect_err("an unparseable version must fail the check");
        assert!(error.contains("unparseable SDK version"), "{error}");
    }
}
