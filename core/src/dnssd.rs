use crate::config::DeviceConfig;
use log::{error, info};
use std::thread;
use std::time::Duration;
use zeroconf::event_loop::TEventLoop;
use zeroconf::service::TMdnsService;
use zeroconf::txt_record::TTxtRecord;
use zeroconf::{MdnsService, ServiceType, TxtRecord};

pub fn serve_dnssd(device_config: &DeviceConfig, port: u16, type_name: &str) {
    let device_config = device_config.clone();
    let type_name = type_name.to_string();
    thread::spawn(move || {
        if let Err(e) = serve_dnssd_thread(&device_config, port, type_name.as_str()) {
            error!("Failed to serve DNS-SD for {}: {}", device_config.name, e);
        }
    });
}

fn serve_dnssd_thread(
    device_config: &DeviceConfig,
    port: u16,
    type_name: &str,
) -> anyhow::Result<()> {
    let sub_types = vec!["universal"];
    let service_type = ServiceType::with_sub_types(type_name, "tcp", sub_types)?;
    let mut service = MdnsService::new(service_type, port);
    let mut txt_record = TxtRecord::new();
    txt_record.insert("txtvers", "1")?;
    txt_record.insert("qtotal", "1")?;
    txt_record.insert(
        "rp",
        device_config
            .basepath
            .as_str()
            .strip_prefix('/')
            .unwrap_or(device_config.basepath.as_str()),
    )?;
    txt_record.insert("ty", device_config.make_and_model.as_str())?;
    txt_record.insert("priority", "0")?;
    let mut pdl = Vec::<&str>::new();
    if cfg!(any(feature = "winpdf", feature = "pdfium")) {
        pdl.push("application/pdf");
    }
    pdl.push("application/vnd.ms-xpsdocument");
    pdl.push("image/pwg-raster");
    pdl.push("image/urf");
    txt_record.insert("pdl", pdl.join(",").as_str())?;
    txt_record.insert("note", "")?;
    txt_record.insert("UUID", device_config.uuid.hyphenated().to_string().as_str())?;
    service.set_name(device_config.name.as_str());
    service.set_txt_record(txt_record);
    service.set_context(Box::new((
        type_name.to_string(),
        device_config.name.clone(),
    )));
    service.set_registered_callback(Box::new(|result, context| {
        let context = context.unwrap();
        let (type_name, device_name) = context.downcast_ref::<(String, String)>().unwrap();
        match result {
            Ok(_) => info!("DNS-SD registered for {} {}", type_name, device_name),
            Err(e) => error!(
                "Failed to register DNS-SD for {} {}: {}",
                type_name, device_name, e
            ),
        }
    }));
    let event_loop = service.register()?;
    loop {
        event_loop.poll(Duration::from_secs(0x7fffffff))?;
    }
}
