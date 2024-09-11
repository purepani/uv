use distribution_types::{
    BuiltDist, CachedDist, Dist, DistributionMetadata, IndexLocations, InstalledMetadata, Name,
    Resolution, ResolvedDist, VersionOrUrlRef,
};
use pep440_rs::Version;
use pep508_rs::MarkerEnvironment;
use pypi_types::{DirectUrl, HashDigest, Metadata, ParsedUrl, Requirement, RequirementSource};
use serde::{Deserialize, Serialize};
use url::Url;
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

impl PipReport {
    pub fn from_resolution(
        dists: &[CachedDist],
        requirements: &[Requirement],
        //resolution: Resolution,
    ) -> PipReport {
        PipReport {
            install: dists
                .iter()
                .map(|dist| {
                    let requirement = requirements
                        .iter()
                        .find(|requirement| requirement.name == *dist.name());

                    return InstallationReportItem {
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
                        metadata: dist.metadata().expect("Failed to Parse"),
                    };
                })
                .collect::<Vec<_>>(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
