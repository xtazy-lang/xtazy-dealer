use std::num::NonZeroU32;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedGitRev {
    Tag(String),
    Branch(String),
}

pub(crate) fn materialize_git_source(
    url: &str,
    reference: &ResolvedGitRev,
    dest: &Path,
) -> Result<(), String> {
    let mut prepare = gix::prepare_clone(url, dest)
        .map_err(|e| format!("gix prepare clone failed for {url}: {e}"))?;

    let ref_name = match reference {
        ResolvedGitRev::Tag(tag) => format!("refs/tags/{}", tag),
        ResolvedGitRev::Branch(branch) => format!("refs/heads/{}", branch),
    };

    let partial_name = <&gix::refs::PartialNameRef>::try_from(ref_name.as_str())
        .map_err(|e| format!("invalid reference name: {e}"))?;

    prepare = prepare
        .with_ref_name(Some(partial_name))
        .map_err(|e| format!("gix with_ref_name failed: {e}"))?;

    prepare = prepare.with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(
        NonZeroU32::new(1).unwrap(),
    ));

    let (mut checkout, _) = prepare
        .fetch_then_checkout(
            gix::progress::Discard,
            &std::sync::atomic::AtomicBool::default(),
        )
        .map_err(|e| format!("gix fetch_then_checkout failed: {e}"))?;

    let _repo = checkout
        .main_worktree(
            gix::progress::Discard,
            &std::sync::atomic::AtomicBool::default(),
        )
        .map_err(|e| format!("gix main_worktree failed: {e}"))?;

    Ok(())
}

pub(crate) fn list_remote_tags(url: &str) -> Result<Vec<String>, String> {
    let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let repo = gix::init_bare(temp_dir.path()).map_err(|e| e.to_string())?;
    let remote = repo.remote_at(url).map_err(|e| e.to_string())?;
    let connection = remote
        .connect(gix::remote::Direction::Fetch)
        .map_err(|e| e.to_string())?;

    let ref_map = connection
        .ref_map(
            gix::progress::Discard,
            gix::remote::ref_map::Options::default(),
        )
        .map_err(|e| e.to_string())?;

    let mut tags = Vec::new();
    for reference in ref_map.remote_refs {
        let (ref_name, _, _) = reference.unpack();
        tags.push(ref_name.to_string());
    }

    Ok(tags)
}
