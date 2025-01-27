use super::ipp_sys_predefined_map::IppSysPredefinedMap;
use winprint::ticket::{JobDuplex, PredefinedDuplexType};

const SIDES_PAIR: &[(&str, PredefinedDuplexType)] = &[
    ("one-sided", PredefinedDuplexType::OneSided),
    (
        "two-sided-long-edge",
        PredefinedDuplexType::TwoSidedLongEdge,
    ),
    (
        "two-sided-short-edge",
        PredefinedDuplexType::TwoSidedShortEdge,
    ),
];

pub struct JobSidesMap;

impl IppSysPredefinedMap for JobSidesMap {
    type IppKey = &'static str;
    type SysPredefined = PredefinedDuplexType;
    type SysOptionPack = JobDuplex;

    fn bimap() -> &'static bimap::BiHashMap<Self::IppKey, Self::SysPredefined> {
        static BIMAP: std::sync::OnceLock<bimap::BiHashMap<&str, PredefinedDuplexType>> =
            std::sync::OnceLock::new();
        BIMAP.get_or_init(|| bimap::BiHashMap::from_iter(SIDES_PAIR.iter().copied()))
    }
}
