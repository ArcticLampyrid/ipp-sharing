use crate::attr::ipp_sys_predefined_map::IppSysPredefinedMap;
use crate::attr::media::get_media_by_ipp;
use crate::attr::orientation::OrientationMap;
use crate::attr::print_color_mode::PrintColorMap;
use crate::attr::printer_resolution::get_resolution_by_ipp;
use crate::attr::sides::JobSidesMap;
use crate::print_options::PrintOptions;
use crate::raster::{cups_raster_to_tiff, urf_to_tiff};
use anyhow::Ok;
use futures::io::{self, BufReader};
use futures::AsyncReadExt;
use ippper::error::IppError;
use ippper::service::simple::{SimpleIppDocument, SimpleIppServiceHandler};
use log::{error, info};
use std::env;
use std::path::Path;
use tokio::fs;
use tokio::fs::File;
use tokio_util::compat::*;
use uuid::Uuid;
use winprint::printer::{FilePrinter, ImagePrinter, PrinterDevice, XpsPrinter};
use winprint::ticket::PrintCapabilities;

pub struct MyHandler {
    target: PrinterDevice,
    capabilities: PrintCapabilities,
}

impl MyHandler {
    pub fn new(target: PrinterDevice, capabilities: PrintCapabilities) -> Self {
        Self {
            target,
            capabilities,
        }
    }
    #[allow(unreachable_code)]
    pub fn handle_pdf(
        target: PrinterDevice,
        path: &Path,
        options: PrintOptions,
    ) -> anyhow::Result<()> {
        info!("Printing PDF document...");
        let ticket = options.into_ticket(&target)?;
        #[cfg(feature = "pdfium")]
        {
            use winprint::printer::PdfiumPrinter;
            let pdf = PdfiumPrinter::new(target);
            pdf.print(path, ticket)?;
            return Ok(());
        }
        #[cfg(feature = "winpdf")]
        {
            use winprint::printer::WinPdfPrinter;
            let pdf = WinPdfPrinter::new(target);
            pdf.print(path, ticket)?;
            return Ok(());
        }
        error!("PDF printing is not supported");
        Err(anyhow::anyhow!("PDF printing is not supported"))
    }
    pub fn handle_xps(
        target: PrinterDevice,
        path: &Path,
        options: PrintOptions,
    ) -> anyhow::Result<()> {
        info!("Printing XPS document...");
        let ticket = options.into_ticket(&target)?;
        let xps = XpsPrinter::new(target);
        xps.print(path, ticket)?;
        Ok(())
    }
    pub fn handle_image(
        target: PrinterDevice,
        path: &Path,
        options: PrintOptions,
    ) -> anyhow::Result<()> {
        info!("Printing image document...");
        let ticket = options.into_ticket(&target)?;
        let image = ImagePrinter::new(target);
        image.print(path, ticket)?;
        Ok(())
    }
}

const RASTER_BUF_SIZE: usize = 1024 * 1024;

impl SimpleIppServiceHandler for MyHandler {
    async fn handle_document(&self, document: SimpleIppDocument) -> anyhow::Result<()> {
        info!(
            "Receiving document from user: {}",
            document.job_attributes.originating_user_name
        );

        let media = get_media_by_ipp(
            self.capabilities.page_media_sizes(),
            document.job_attributes.media.as_str(),
        );
        let orientation = document
            .job_attributes
            .orientation
            .and_then(|x| OrientationMap::find_by_ipp(self.capabilities.page_orientations(), &x));
        let output_color = PrintColorMap::find_by_ipp(
            self.capabilities.page_output_colors(),
            document.job_attributes.print_color_mode.as_str(),
        );
        let job_duplex = JobSidesMap::find_by_ipp(
            self.capabilities.duplexes(),
            document.job_attributes.sides.as_str(),
        );
        let resolution = document
            .job_attributes
            .printer_resolution
            .and_then(|x| get_resolution_by_ipp(self.capabilities.page_resolutions(), &x));
        let options = PrintOptions {
            media,
            orientation,
            output_color,
            job_duplex,
            resolution,
        };
        let mut header = [0u8; 4];
        let mut payload = document.payload;
        payload.read_exact(&mut header).await?;
        let mut payload = futures::io::Cursor::new(header).chain(payload);
        let target = self.target.clone();

        let mut path = env::temp_dir();
        path.push(Uuid::new_v4().simple().to_string());
        if &header == b"%PDF" {
            path.set_extension("pdf");
            let mut file = File::create(&path).await?.compat();
            #[allow(clippy::never_loop)]
            let r = loop {
                if let Err(err) = io::copy(&mut payload, &mut file).await {
                    error!("Failed to save document as file: {:#}", err);
                    break Err(err.into());
                }
                drop(file);
                if let Err(err) = blocking::unblock({
                    let path = path.clone();
                    move || Self::handle_pdf(target, &path, options)
                })
                .await
                {
                    error!("Failed to print document: {:#}", err);
                    break Err(err);
                }
                break Ok(());
            };
            let _ = fs::remove_file(&path).await;
            r
        } else if &header == b"PK\x03\x04" {
            path.set_extension("xps");
            let mut file = File::create(&path).await?.compat();
            #[allow(clippy::never_loop)]
            let r = loop {
                if let Err(err) = io::copy(&mut payload, &mut file).await {
                    error!("Failed to save document as file: {:#}", err);
                    break Err(err.into());
                }
                drop(file);
                if let Err(err) = blocking::unblock({
                    let path = path.clone();
                    move || Self::handle_xps(target, &path, options)
                })
                .await
                {
                    error!("Failed to print document: {:#}", err);
                    break Err(err);
                }
                break Ok(());
            };
            let _ = fs::remove_file(&path).await;
            r
        } else if &header == b"UNIR" {
            path.set_extension("tiff");
            #[allow(clippy::never_loop)]
            let r = loop {
                if let Err(err) =
                    urf_to_tiff(BufReader::with_capacity(RASTER_BUF_SIZE, payload), &path).await
                {
                    error!("Failed to save document as file: {:#}", err);
                    break Err(err);
                }
                if let Err(err) = blocking::unblock({
                    let path = path.clone();
                    move || Self::handle_image(target, &path, options)
                })
                .await
                {
                    error!("Failed to print document: {:#}", err);
                    break Err(err);
                }
                break Ok(());
            };
            let _ = fs::remove_file(&path).await;
            r
        } else if &header == b"RaSt"
            || &header == b"tSaR"
            || &header == b"RaS2"
            || &header == b"2SaR"
            || &header == b"RaS3"
            || &header == b"3Sar"
        {
            path.set_extension("tiff");
            #[allow(clippy::never_loop)]
            let r = loop {
                if let Err(err) =
                    cups_raster_to_tiff(BufReader::with_capacity(RASTER_BUF_SIZE, payload), &path)
                        .await
                {
                    error!("Failed to save document as file: {:#}", err);
                    break Err(err);
                }
                if let Err(err) = blocking::unblock({
                    let path = path.clone();
                    move || Self::handle_image(target, &path, options)
                })
                .await
                {
                    error!("Failed to print document: {:#}", err);
                    break Err(err);
                }
                break Ok(());
            };
            let _ = fs::remove_file(&path).await;
            r
        } else {
            error!("Unsupported document format, header: {:?}", header);
            Err(IppError {
                code: ipp::model::StatusCode::ClientErrorDocumentFormatNotSupported,
                msg: ipp::model::StatusCode::ClientErrorDocumentFormatNotSupported.to_string(),
            }
            .into())
        }
    }
}
