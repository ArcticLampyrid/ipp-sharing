use anyhow::Ok;
use blocking::Task;
use futures::{AsyncRead, AsyncReadExt};
use print_raster::{
    model::{
        cups::{CupsColorOrder, CupsColorSpace},
        urf::UrfColorSpace,
    },
    reader::{
        cups::unified::CupsRasterUnifiedReader, urf::UrfReader, RasterPageReader, RasterReader,
    },
};
use std::{path::Path, pin::pin};
use tiff::{
    encoder::{
        colortype::{Gray8, RGB8},
        compression::DeflateLevel,
        Compression, Rational, TiffEncoder,
    },
    tags::ResolutionUnit,
};
use tokio::sync::mpsc;

enum ContentFormat {
    Gray8,
    RGB8,
}

struct OwnedPage {
    width: u32,
    height: u32,
    dpi_x: u32,
    dpi_y: u32,
    format: ContentFormat,
    content: Vec<u8>,
}

fn new_tiff_encode_task(
    tiff_path: &Path,
) -> anyhow::Result<(mpsc::Sender<OwnedPage>, Task<anyhow::Result<()>>)> {
    let tiff_file = std::fs::File::create(tiff_path)?;
    let (tx, mut rx) = mpsc::channel::<OwnedPage>(3);
    let tiff_encode_task = blocking::unblock(move || {
        let mut encoder = TiffEncoder::new(tiff_file)?
            .with_compression(Compression::Deflate(DeflateLevel::Balanced));
        while let Some(page) = rx.blocking_recv() {
            match page.format {
                ContentFormat::Gray8 => {
                    let mut image = encoder.new_image::<Gray8>(page.width, page.height)?;
                    image.resolution_unit(ResolutionUnit::Inch);
                    image.x_resolution(Rational {
                        n: page.dpi_x,
                        d: 1,
                    });
                    image.y_resolution(Rational {
                        n: page.dpi_y,
                        d: 1,
                    });
                    image.write_data(&page.content)?;
                }
                ContentFormat::RGB8 => {
                    let mut image = encoder.new_image::<RGB8>(page.width, page.height)?;
                    image.resolution_unit(ResolutionUnit::Inch);
                    image.x_resolution(Rational {
                        n: page.dpi_x,
                        d: 1,
                    });
                    image.y_resolution(Rational {
                        n: page.dpi_y,
                        d: 1,
                    });
                    image.write_data(&page.content)?;
                }
            }
        }
        Ok(())
    });
    Ok((tx, tiff_encode_task))
}

pub async fn urf_to_tiff<R>(raster: R, tiff_path: &Path) -> anyhow::Result<()>
where
    R: AsyncRead,
{
    let (tx, tiff_encode_task) = new_tiff_encode_task(tiff_path)?;
    let raster = pin!(raster);
    let reader = UrfReader::new(raster).await?;
    let mut page_next = reader.next_page().await?;
    while let Some(mut page) = page_next {
        let mut data = Vec::<u8>::new();
        page.content_mut().read_to_end(&mut data).await?;
        tx.send(OwnedPage {
            width: page.header().width,
            height: page.header().height,
            dpi_x: page.header().dot_per_inch,
            dpi_y: page.header().dot_per_inch,
            format: match (page.header().color_space, page.header().bits_per_pixel) {
                (UrfColorSpace::sGray, 8) | (UrfColorSpace::Gray, 8) => ContentFormat::Gray8,
                (UrfColorSpace::sRGB, 24) | (UrfColorSpace::RGB, 24) => ContentFormat::RGB8,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported pixel data: {:?} ({} bits per pixel)",
                        page.header().color_space,
                        page.header().bits_per_pixel
                    ))
                }
            },
            content: data,
        })
        .await?;
        page_next = page.next_page().await?;
    }
    drop(tx);
    tiff_encode_task.await?;
    Ok(())
}

pub async fn cups_raster_to_tiff<R>(raster: R, tiff_path: &Path) -> anyhow::Result<()>
where
    R: AsyncRead,
{
    let (tx, tiff_encode_task) = new_tiff_encode_task(tiff_path)?;
    let raster = pin!(raster);
    let reader = CupsRasterUnifiedReader::new(raster).await?;
    let mut page_next = reader.next_page().await?;
    while let Some(mut page) = page_next {
        if page.header().v1.color_order != CupsColorOrder::Chunky {
            return Err(anyhow::anyhow!(
                "Unsupported color order: {:?}",
                page.header().v1.color_order
            ));
        }
        let mut data = Vec::<u8>::new();
        page.content_mut().read_to_end(&mut data).await?;
        tx.send(OwnedPage {
            width: page.header().v1.width,
            height: page.header().v1.height,
            dpi_x: page.header().v1.resolution.cross_feed,
            dpi_y: page.header().v1.resolution.feed,
            format: match (
                page.header().v1.color_space,
                page.header().v1.bits_per_pixel,
            ) {
                (CupsColorSpace::sGray, 8) | (CupsColorSpace::Gray, 8) => ContentFormat::Gray8,
                (CupsColorSpace::sRGB, 24) | (CupsColorSpace::RGB, 24) => ContentFormat::RGB8,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported pixel data: {:?} ({} bits per pixel)",
                        page.header().v1.color_space,
                        page.header().v1.bits_per_pixel
                    ))
                }
            },
            content: data,
        })
        .await?;
        page_next = page.next_page().await?;
    }
    drop(tx);
    tiff_encode_task.await?;
    Ok(())
}
