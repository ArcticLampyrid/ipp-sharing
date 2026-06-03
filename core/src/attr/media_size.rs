use ippper::model::MediaSize;
use winprint::ticket::MediaSizeTuple;

pub fn media_size_sys_to_ipp(size: MediaSizeTuple) -> MediaSize {
    let x_in_unit = (size.width_in_micron() + 5) / 10;
    let y_in_unit = (size.height_in_micron() + 5) / 10;
    MediaSize {
        x_dimension: x_in_unit.try_into().unwrap_or(0),
        y_dimension: y_in_unit.try_into().unwrap_or(0),
    }
}
