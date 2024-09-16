use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Display;

use distribution_types::{
    BuiltDist, DirectorySourceDist, Dist, DistributionMetadata, IndexUrl, InstalledDist, Name,
    ResolvedDist, SourceDist, VersionOrUrlRef,
};
use pep440_rs::Version;
use pep508_rs::MarkerTree;
use pypi_types::{ArchiveInfo, DirInfo, DirectUrl, HashDigest, VcsInfo, VcsKind};
use url::Url;
use uv_distribution;
use uv_git::GitReference;
use uv_normalize::{ExtraName, GroupName, PackageName};

pub use crate::resolution::display::{AnnotationStyle, DisplayResolutionGraph};
pub use crate::resolution::graph::ResolutionGraph;
pub(crate) use crate::resolution::graph::ResolutionGraphNode;
pub(crate) use crate::resolution::requirements_txt::RequirementsTxtDist;

mod display;
mod graph;
mod requirements_txt;

/// A pinned package with its resolved distribution and metadata. The [`ResolvedDist`] refers to a
/// specific distribution (e.g., a specific wheel), while the [`Metadata23`] refers to the metadata
/// for the package-version pair.
#[derive(Debug, Clone)]
pub(crate) struct AnnotatedDist {
    pub(crate) dist: ResolvedDist,
    pub(crate) name: PackageName,
    pub(crate) version: Version,
    pub(crate) extra: Option<ExtraName>,
    pub(crate) dev: Option<GroupName>,
    pub(crate) hashes: Vec<HashDigest>,
    pub(crate) metadata: Option<uv_distribution::Metadata>,
    pub(crate) metadata_full: Option<pypi_types::Metadata>,
    pub(crate) marker: MarkerTree,
}

impl AnnotatedDist {
    /// Returns `true` if the [`AnnotatedDist`] is a base package (i.e., not an extra or a
    /// dependency group).
    pub(crate) fn is_base(&self) -> bool {
        self.extra.is_none() && self.dev.is_none()
    }

    /// Returns the [`IndexUrl`] of the distribution, if it is from a registry.
    pub(crate) fn index(&self) -> Option<&IndexUrl> {
        match &self.dist {
            ResolvedDist::Installed(_) => None,
            ResolvedDist::Installable(dist) => match dist {
                Dist::Built(dist) => match dist {
                    BuiltDist::Registry(dist) => Some(&dist.best_wheel().index),
                    BuiltDist::DirectUrl(_) => None,
                    BuiltDist::Path(_) => None,
                },
                Dist::Source(dist) => match dist {
                    SourceDist::Registry(dist) => Some(&dist.index),
                    SourceDist::DirectUrl(_) => None,
                    SourceDist::Git(_) => None,
                    SourceDist::Path(_) => None,
                    SourceDist::Directory(_) => None,
                },
            },
        }
    }

    pub(crate) fn url(&self) -> Result<Url, ()> {
        match &self.dist {
            ResolvedDist::Installed(dist) => match dist {
                InstalledDist::Registry(dist) => Url::from_file_path(&dist.path),
                InstalledDist::Url(dist) => Ok(dist.url.clone()),
                InstalledDist::EggInfoFile(dist) => Url::from_file_path(&dist.path),
                InstalledDist::EggInfoDirectory(dist) => Url::from_file_path(&dist.path),
                InstalledDist::LegacyEditable(dist) => Ok(dist.target_url.clone()),
            },
            ResolvedDist::Installable(dist) => match dist {
                Dist::Built(dist) => match dist {
                    BuiltDist::Registry(dist) => Ok(dist.best_wheel().index.url().clone()),
                    BuiltDist::DirectUrl(dist) => Ok(dist.location.clone()),
                    BuiltDist::Path(dist) => Url::from_file_path(&dist.install_path),
                },
                Dist::Source(dist) => match dist {
                    SourceDist::Registry(dist) => Ok(dist.index.url().clone()),
                    SourceDist::DirectUrl(dist) => Ok(dist.location.clone()),
                    SourceDist::Git(dist) => Ok(dist.git.repository().clone()),
                    SourceDist::Path(dist) => Url::from_file_path(&dist.install_path),
                    SourceDist::Directory(dist) => Url::from_file_path(&dist.install_path),
                },
            },
        }
    }

    pub fn direct_url(&self) -> Option<DirectUrl> {
        let url = self.url().ok()?.to_string();
        let mut hashes = BTreeMap::new();
        for hash in self.hashes.clone() {
            hashes.insert(hash.algorithm.to_string(), hash.digest.to_string());
        }

        let subdirectory = match &self.dist {
            ResolvedDist::Installable(Dist::Source(SourceDist::DirectUrl(dist))) => {
                dist.subdirectory.clone()
            }
            ResolvedDist::Installable(Dist::Source(SourceDist::Git(dist))) => {
                dist.subdirectory.clone()
            }
            _ => None,
        };

        match &self.dist {
            ResolvedDist::Installed(InstalledDist::EggInfoFile(_))
            | ResolvedDist::Installed(InstalledDist::EggInfoDirectory(_))
            | ResolvedDist::Installed(InstalledDist::LegacyEditable(_))
            | ResolvedDist::Installable(Dist::Built(BuiltDist::Path(_)))
            | ResolvedDist::Installable(Dist::Source(SourceDist::Path(_)))
            | ResolvedDist::Installable(Dist::Source(SourceDist::Directory(_))) => {
                Some(DirectUrl::LocalDirectory {
                    url,
                    dir_info: DirInfo {
                        editable: Some(self.dist.is_editable().clone()),
                    },
                })
            }
            ResolvedDist::Installed(InstalledDist::Registry(_))
            | ResolvedDist::Installed(InstalledDist::Url(_))
            | ResolvedDist::Installable(Dist::Built(BuiltDist::Registry(_)))
            | ResolvedDist::Installable(Dist::Built(BuiltDist::DirectUrl(_)))
            | ResolvedDist::Installable(Dist::Source(SourceDist::Registry(_)))
            | ResolvedDist::Installable(Dist::Source(SourceDist::DirectUrl(_))) => {
                Some(DirectUrl::ArchiveUrl {
                    url,
                    archive_info: ArchiveInfo {
                        hash: None,
                        hashes: Some(hashes),
                    },
                    subdirectory,
                })
            }
            ResolvedDist::Installable(Dist::Source(SourceDist::Git(dist))) => {
                Some(DirectUrl::VcsUrl {
                    url,
                    vcs_info: VcsInfo {
                        vcs: VcsKind::Git,
                        commit_id: match dist.git.precise() {
                            Some(git_sha) => Some(git_sha.to_string()),
                            None => None,
                        },
                        requested_revision: Some(dist.git.reference().as_str()?.to_string()),
                    },
                    subdirectory,
                })
            }
        }
    }
}

impl Name for AnnotatedDist {
    fn name(&self) -> &PackageName {
        self.dist.name()
    }
}

impl DistributionMetadata for AnnotatedDist {
    fn version_or_url(&self) -> VersionOrUrlRef {
        self.dist.version_or_url()
    }
}

impl Display for AnnotatedDist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.dist, f)
    }
}
