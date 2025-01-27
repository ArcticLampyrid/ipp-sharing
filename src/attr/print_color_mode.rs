use super::ipp_sys_predefined_map::IppSysPredefinedMap;
use winprint::ticket::{PageOutputColor, PredefinedPageOutputColor};

const PRINT_COLOR_PAIR: &[(&str, PredefinedPageOutputColor)] = &[
    ("monochrome", PredefinedPageOutputColor::Monochrome),
    ("grayscale", PredefinedPageOutputColor::Grayscale),
    ("color", PredefinedPageOutputColor::Color),
];

pub struct PrintColorMap;

impl IppSysPredefinedMap for PrintColorMap {
    type IppKey = &'static str;
    type SysPredefined = PredefinedPageOutputColor;
    type SysOptionPack = PageOutputColor;

    fn bimap() -> &'static bimap::BiHashMap<Self::IppKey, Self::SysPredefined> {
        static BIMAP: std::sync::OnceLock<bimap::BiHashMap<&str, PredefinedPageOutputColor>> =
            std::sync::OnceLock::new();
        BIMAP.get_or_init(|| bimap::BiHashMap::from_iter(PRINT_COLOR_PAIR.iter().copied()))
    }
}
