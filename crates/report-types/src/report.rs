use distribution_types::{
    BuiltDist, CachedDist, Dist, DistributionMetadata, IndexLocations, InstalledMetadata, Name,
    Resolution, ResolvedDist, VersionOrUrlRef,
};
use itertools::Itertools;
use pep440_rs::Version;
use pep508_rs::MarkerEnvironment;
use pypi_types::{DirectUrl, HashDigest, Metadata, ParsedUrl, Requirement, RequirementSource};
use serde::{Deserialize, Serialize};
use url::Url;
use uv_metadata::read_flat_wheel_metadata_full;
use uv_normalize::{ExtraName, PackageName};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PipReport {
    //pub version: String,
    //pub pip_version: String,
    pub install: Vec<InstallationReportItem>,
    //pub environment: MarkerEnvironment,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct InstallationReportItem {
    pub metadata: Metadata,
    pub is_direct: bool,
    pub is_yanked: bool,
    pub download_info: Option<DirectUrl>,
    pub requested: bool,
    pub requested_extras: Vec<ExtraName>,
}

impl InstallationReportItem {
    pub fn from_cached_dist(
        dist: ResolvedDist,
        requirements: &[Requirement],
    ) -> InstallationReportItem {
        let requirement = requirements
            .iter()
            .find(|requirement| requirement.name == *dist.name());

        InstallationReportItem {
            is_direct: match dist.version_or_url() {
                VersionOrUrlRef::Url(_) => true,
                _ => false,
            },
            is_yanked: false,
            download_info: match requirement {
                Some(req) => None,
                None => None,
            },
            requested: match requirement {
                Some(req) => true,
                None => false,
            },
            requested_extras: match requirement {
                Some(req) => req.extras.clone(),
                None => vec![],
            },
            metadata: read_flat_wheel_metadata_full(dist.filename(), dist.path().into_iter())
                .unwrap(),
        }
    }
}

impl PipReport {
    pub fn from_resolution(
        resolution: Resolution,
        dists: &[CachedDist],
        requirements: &[Requirement],
        //resolution: Resolution,
    ) -> PipReport {
        resolution.requirements()
        let install = dists
            .iter()
            .map(|dist| InstallationReportItem::from_cached_dist(dist, requirements))
            .collect();
        PipReport { install }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
