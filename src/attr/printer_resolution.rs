use ippper::model::Resolution as IppResolution;
use winprint::ticket::PageResolution;

pub fn all_supported_resolution_by_win(
    packs: impl Iterator<Item = PageResolution>,
) -> Vec<IppResolution> {
    packs
        .filter_map(|r| {
            let (x, y) = r.dpi();
            let (x, y) = (x.try_into().ok()?, y.try_into().ok()?);
            Some(IppResolution::new_dpi(x, y))
        })
        .collect()
}

pub fn get_resolution_by_ipp(
    mut packs: impl Iterator<Item = PageResolution>,
    ipp_key: &IppResolution,
) -> Option<PageResolution> {
    if ipp_key.units != 3 {
        return None;
    }
    packs.find(|r| {
        let (x, y) = r.dpi();
        match (x.try_into(), y.try_into()) {
            (Ok(x), Ok(y)) => ipp_key.cross_feed == x && ipp_key.feed == y,
            _ => false,
        }
    })
}
