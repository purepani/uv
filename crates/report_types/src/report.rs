use distribution_types::{
    BuiltDist, Dist, DistributionMetadata, IndexLocations, InstalledMetadata, Name, Resolution, ResolvedDist
};
use pypi_types::{HashDigest, Requirement, RequirementSource};

struct PipReport {
    version: String,
    install: Vec<InstallationReportItem>,
    environment: ,
}

struct InstallationReportItem {
    metadata: ,
    is_direct: bool,
    is_yanked: bool,
    download_info: ,
    requested: bool,
    requested_extras: ,
}

impl InstallationReportItem {
    fn from_dist(resolved_dist: &ResolvedDist) {
        let is_editable = resolved_dist.is_editable();
        let is_direct = match resolved_dist {
            ResolvedDist::Installable(dist) => match dist {
                Dist::Built(BuiltDist::DirectUrl) => true,
                Dist::Source(SourceDist::DirectUrl) => true,
                _ => false,
            },
            _ => false,
        };
        let is_yanked = resolved_dist.yanked();
        let requested = resolved_dist.requested;
    }
}

