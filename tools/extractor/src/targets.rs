use anyhow::Result;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use wax::Glob;

pub fn find(paths: &[PathBuf], prefix: Option<&Path>, files: &[String]) -> Result<Vec<PathBuf>> {
    let result = files
        .par_iter()
        .map(|file| descend(prefix, paths, file))
        .collect::<Result<Vec<_>>>()?;

    let mut buckets: HashMap<String, Vec<Candidate>> = HashMap::new();

    for candidate in result.into_iter().flatten() {
        buckets
            .entry(candidate.key.to_string())
            .or_default()
            .push(candidate);
    }

    Ok(buckets
        .into_values()
        .map(|v| v.into_iter().max_by_key(|v| v.priority).unwrap().path)
        .collect())
}

fn descend(prefix: Option<&Path>, paths: &[PathBuf], file: &str) -> Result<Vec<Candidate>> {
    let result = paths
        .par_iter()
        .enumerate()
        .map(|(index, path)| walk(paths.len() - index, prefix, path, file))
        .collect::<Result<Vec<_>>>()?;

    Ok(result.into_iter().flatten().collect())
}

fn walk(priority: usize, prefix: Option<&Path>, path: &Path, glob: &str) -> Result<Vec<Candidate>> {
    let path = match prefix {
        Some(v) => &path.join(v),
        None => path,
    };

    if !path.exists() {
        return Ok(Vec::new());
    }

    let glob = Glob::new(glob)?;

    Ok(glob
        .walk(path)
        .flat_map(|v| -> Option<_> {
            // Glob should only fail if the path cannot be found, in which case we just assume the
            // pak we're scanning does not contain any of the files we're searching for.
            let item = match v {
                Ok(v) => v.into_path(),
                Err(_) => return None,
            };

            let key = item
                .strip_prefix(path)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            Some(Candidate {
                key,
                path: item,
                priority,
            })
        })
        .collect())
}

#[derive(Debug)]
pub struct Candidate {
    pub key: String,
    pub path: PathBuf,
    pub priority: usize,
}
