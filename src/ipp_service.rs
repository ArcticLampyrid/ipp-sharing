use crate::attr::ipp_sys_predefined_map::IppSysPredefinedMap;
use crate::attr::media::all_supported_media;
use crate::attr::orientation::OrientationMap;
use crate::attr::print_color_mode::PrintColorMap;
use crate::attr::printer_resolution::all_supported_resolution_by_win;
use crate::attr::sides::JobSidesMap;
use crate::config::{DeviceConfig, OneOrMany, ServerConfig};
use crate::dnssd::serve_dnssd;
use crate::handler::MyHandler;
use gethostname::gethostname;
use ippper::service::simple::{PrinterInfoBuilder, SimpleIppService};
use std::sync::Arc;
use winprint::printer::PrinterDevice;
use winprint::ticket::PrintCapabilities;

pub struct MyIppService {
    pub inner: Arc<SimpleIppService<MyHandler>>,
    /// basepath with a prefixed slash, but without a trailing slash.
    formatted_basepath: String,
}

impl MyIppService {
    pub fn new(
        server_config: &ServerConfig,
        device_config: &DeviceConfig,
    ) -> anyhow::Result<MyIppService> {
        let devices = PrinterDevice::all()?;
        let device = devices
            .into_iter()
            .find(|x| x.name() == device_config.target)
            .ok_or_else(|| anyhow::anyhow!("target printer not found: {}", device_config.target))?;

        let capabilities = PrintCapabilities::fetch(&device)?;

        let mut info = PrinterInfoBuilder::default();
        let mut format_supported = vec!["application/octet-stream".to_string()];
        if cfg!(any(feature = "winpdf", feature = "pdfium")) {
            format_supported.push("application/pdf".to_string());
        }
        format_supported.push("application/vnd.ms-xpsdocument".to_string());
        format_supported.push("image/pwg-raster".to_string());
        format_supported.push("image/urf".to_string());
        let format_preferred = if format_supported.len() > 1 {
            Some(format_supported[1].clone())
        } else {
            None
        };
        info.name(device_config.name.clone())
            .info(Some(device_config.info.clone()))
            .make_and_model(Some(device_config.make_and_model.clone()))
            .document_format_supported(format_supported)
            .document_format_default("application/octet-stream".to_string())
            .document_format_preferred(format_preferred)
            .uuid(Some(device_config.uuid));

        if device_config.dnssd {
            info.dnssd_name(Some(device_config.name.clone()));
        }

        let media_supported = all_supported_media(capabilities.page_media_sizes());
        if media_supported.iter().any(|x| x == "iso_a4_210x297mm") {
            info.media_default("iso_a4_210x297mm".to_string());
        } else if !media_supported.is_empty() {
            info.media_default(media_supported[0].clone());
        }
        info.media_supported(media_supported);

        let orientation_supported =
            OrientationMap::all_supported_by_win(capabilities.page_orientations());
        if !orientation_supported.is_empty() {
            info.orientation_supported(orientation_supported);
        }

        let color_supported =
            PrintColorMap::all_supported_by_win(capabilities.page_output_colors())
                .into_iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
        let color = color_supported.iter().any(|x| x == "color");
        info.color_supported(color);
        if color_supported.iter().any(|x| x == "grayscale") {
            // For environmental friendliness, grayscale mode is used by default.
            info.print_color_mode_default("grayscale".to_string());
            info.print_color_mode_supported(color_supported);
        } else if !color_supported.is_empty() {
            info.print_color_mode_default(color_supported[0].clone());
            info.print_color_mode_supported(color_supported);
        }

        let mut sides_supported = JobSidesMap::all_supported_by_win(capabilities.duplexes())
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        if sides_supported.is_empty() {
            sides_supported.push("one-sided".to_string());
        }
        info.sides_supported(sides_supported);

        let resolution_supported = all_supported_resolution_by_win(capabilities.page_resolutions());
        if !resolution_supported.is_empty() {
            info.printer_resolution_default(Some(resolution_supported[0]));
            info.printer_resolution_supported(resolution_supported.clone());
        }

        info.urf_supported({
            let mut r = vec!["V1.4".to_string(), "CP1".to_string(), "W8".to_string()];
            if color {
                r.push("SRGB24".to_string());
            }
            let mut urf_resolution = resolution_supported
                .iter()
                .map(|x| x.cross_feed.max(x.feed))
                .collect::<Vec<_>>();
            urf_resolution.sort_unstable();
            urf_resolution.dedup();
            r.push(format!(
                "RS{}",
                urf_resolution
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("-")
            ));
            r.push("DM1".to_string());
            r
        });

        info.pwg_raster_document_resolution_supported(resolution_supported);
        info.pwg_raster_document_sheet_back(Some("normal".to_string()));
        info.pwg_raster_document_type_supported({
            let mut r = vec!["sgray_8".to_string()];
            if color {
                r.push("srgb_8".to_string());
            }
            r
        });

        let info = info.build()?;

        let mut ipp_service = SimpleIppService::new(info, MyHandler::new(device, capabilities));
        let port = match &server_config.addr {
            OneOrMany::One(addr) => addr.port(),
            OneOrMany::Many(addrs) => addrs.first().map_or(631, |addr| addr.port()),
        };
        let host = server_config
            .host
            .clone()
            .unwrap_or_else(|| format!("{}.local:{}", gethostname().to_string_lossy(), port));
        ipp_service.set_host(host.as_ref());
        ipp_service.set_basepath(device_config.basepath.as_str());

        if device_config.dnssd {
            serve_dnssd(device_config, port, "ipp");
            if server_config.tls.is_some() {
                serve_dnssd(device_config, port, "ipps");
            }
        }

        let formatted_basepath = device_config.basepath.as_str();
        let formatted_basepath = formatted_basepath
            .strip_prefix('/')
            .unwrap_or(formatted_basepath);
        let formatted_basepath = formatted_basepath
            .strip_suffix('/')
            .unwrap_or(formatted_basepath);
        let formatted_basepath = format!("/{}", formatted_basepath);

        Ok(MyIppService {
            inner: Arc::new(ipp_service),
            formatted_basepath,
        })
    }

    pub fn matches(&self, path: &str) -> bool {
        path.strip_prefix(self.formatted_basepath.as_str())
            .is_some_and(|x| x.starts_with('/') || x.is_empty())
    }
}
