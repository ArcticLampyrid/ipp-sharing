use super::ipp_sys_predefined_map::IppSysPredefinedMap;
use ippper::model::PageOrientation as IppPageOrientation;
use winprint::ticket::{PageOrientation, PredefinedPageOrientation};

const ORIENTATION_PAIR: &[(IppPageOrientation, PredefinedPageOrientation)] = &[
    (
        IppPageOrientation::Portrait,
        PredefinedPageOrientation::Portrait,
    ),
    (
        IppPageOrientation::Landscape,
        PredefinedPageOrientation::Landscape,
    ),
    (
        IppPageOrientation::ReversePortrait,
        PredefinedPageOrientation::ReversePortrait,
    ),
    (
        IppPageOrientation::ReverseLandscape,
        PredefinedPageOrientation::ReverseLandscape,
    ),
];

pub struct OrientationMap;

impl IppSysPredefinedMap for OrientationMap {
    type IppKey = IppPageOrientation;
    type SysPredefined = PredefinedPageOrientation;
    type SysOptionPack = PageOrientation;

    fn bimap() -> &'static bimap::BiHashMap<Self::IppKey, Self::SysPredefined> {
        static BIMAP: std::sync::OnceLock<
            bimap::BiHashMap<IppPageOrientation, PredefinedPageOrientation>,
        > = std::sync::OnceLock::new();
        BIMAP.get_or_init(|| bimap::BiHashMap::from_iter(ORIENTATION_PAIR.iter().copied()))
    }
}
