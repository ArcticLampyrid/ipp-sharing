use std::{borrow::Borrow, hash::Hash};
use winprint::ticket::{FeatureOptionPackWithPredefined, PredefinedName};

pub trait IppSysPredefinedMap {
    type IppKey: Eq + Hash + Clone + 'static;
    type SysPredefined: PredefinedName + Eq + Hash + 'static;
    type SysOptionPack: FeatureOptionPackWithPredefined<PredefinedName = Self::SysPredefined>
        + 'static;

    fn bimap() -> &'static bimap::BiHashMap<Self::IppKey, Self::SysPredefined>;

    fn all_supported_by_win(packs: impl Iterator<Item = Self::SysOptionPack>) -> Vec<Self::IppKey> {
        let map = Self::bimap();
        packs
            .filter_map(|x| x.as_predefined_name())
            .filter_map(|x| map.get_by_right(&x).cloned())
            .collect()
    }

    fn find_by_ipp<Q>(
        mut packs: impl Iterator<Item = Self::SysOptionPack>,
        ipp_key: &Q,
    ) -> Option<Self::SysOptionPack>
    where
        Self::IppKey: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let map = Self::bimap();
        let predefined = map.get_by_left(ipp_key)?;
        packs.find(|x| x.as_predefined_name().is_some_and(|x| &x == predefined))
    }
}
