//! The workspace, as cargo describes it: its crates and where the index goes.

use std::path::Path;

use crate::index::Paths;
use crate::model::CrateInput;

pub struct Workspace {
    pub paths: Paths,
    pub crates: Vec<CrateInput>,
}

/// Reads the workspace with `cargo metadata` (from the current directory, or
/// the given manifest).
pub fn load(manifest_path: Option<&Path>) -> Result<Workspace, String> {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.no_deps();
    if let Some(p) = manifest_path {
        // Absolute first: cargo runs in another directory below, and would read
        // a relative path from there.
        let p = std::path::absolute(p).map_err(|e| format!("{}: {e}", p.display()))?;
        // Cargo reads its configuration (a shared target dir, for one) from the
        // directory it runs in: run it in the workspace's, not in this tool's.
        if let Some(dir) = p.parent() {
            cmd.current_dir(dir);
        }
        cmd.manifest_path(p);
    }
    let meta = cmd.exec().map_err(|e| format!("cargo metadata: {e}"))?;
    let root = meta.workspace_root.clone().into_std_path_buf();
    let members: std::collections::HashSet<_> = meta.workspace_members.iter().collect();
    let mut crates: Vec<CrateInput> = meta
        .packages
        .iter()
        .filter(|p| members.contains(&p.id))
        .filter_map(|p| {
            let dir = p
                .manifest_path
                .parent()?
                .strip_prefix(&meta.workspace_root)
                .ok()?;
            Some(CrateInput {
                package: p.name.to_string(),
                name: p.name.to_string(),
                dir: dir.as_str().replace('\\', "/"),
                description: p.description.clone(),
            })
        })
        .collect();
    let prefix = shared_prefix(crates.iter().map(|c| c.name.as_str()));
    for c in &mut crates {
        c.name = c.name[prefix.len()..].to_string();
    }
    crates.sort_by(|a, b| a.name.cmp(&b.name));
    let target = meta.target_directory.clone().into_std_path_buf();
    Ok(Workspace {
        paths: Paths::new(root, &target),
        crates,
    })
}

/// The prefix every crate name shares up to a `-` (`acme-engine`, `acme-model`
/// share `acme-`), so the guide can call them `engine` and `model`. Nothing
/// when there is one crate, or when the names share no such prefix: a name is
/// never shortened to nothing.
fn shared_prefix<'a>(names: impl Iterator<Item = &'a str>) -> String {
    let names: Vec<&str> = names.collect();
    let [first, rest @ ..] = names.as_slice() else {
        return String::new();
    };
    if rest.is_empty() {
        return String::new();
    }
    let mut prefix = String::new();
    for (i, _) in first.match_indices('-') {
        let candidate = &first[..=i];
        if names
            .iter()
            .all(|n| n.starts_with(candidate) && n.len() > candidate.len())
        {
            prefix = candidate.to_string();
        } else {
            break;
        }
    }
    prefix
}

#[cfg(test)]
mod tests {
    use super::shared_prefix;

    #[test]
    fn a_shared_dash_prefix_is_dropped_from_crate_names() {
        let names = ["acme-engine", "acme-law-model", "acme-shared"];
        assert_eq!(shared_prefix(names.into_iter()), "acme-");
    }

    #[test]
    fn nothing_is_dropped_without_a_shared_prefix_or_from_a_single_crate() {
        assert_eq!(shared_prefix(["engine", "model"].into_iter()), "");
        assert_eq!(
            shared_prefix(["acme-engine", "other-model"].into_iter()),
            ""
        );
        assert_eq!(shared_prefix(["acme-engine"].into_iter()), "");
        // `acme` alone would become an empty name.
        assert_eq!(shared_prefix(["acme-", "acme-x"].into_iter()), "");
    }

    #[test]
    fn the_prefix_stops_at_the_last_dash_all_names_share() {
        let names = ["acme-tools-a", "acme-tools-b"];
        assert_eq!(shared_prefix(names.into_iter()), "acme-tools-");
    }
}
