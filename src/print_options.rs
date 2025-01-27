use winprint::{
    printer::PrinterDevice,
    ticket::{
        JobDuplex, PageMediaSize, PageOrientation, PageOutputColor, PageResolution, PrintTicket,
        PrintTicketBuilder,
    },
};

#[derive(Debug, Clone)]
pub struct PrintOptions {
    pub media: Option<PageMediaSize>,
    pub orientation: Option<PageOrientation>,
    pub output_color: Option<PageOutputColor>,
    pub job_duplex: Option<JobDuplex>,
    pub resolution: Option<PageResolution>,
}

impl PrintOptions {
    pub fn into_ticket(self, device: &PrinterDevice) -> anyhow::Result<PrintTicket> {
        let mut ticket_builder = PrintTicketBuilder::new(device)?;
        if let Some(media) = self.media {
            ticket_builder.merge(media)?;
        }
        if let Some(orientation) = self.orientation {
            ticket_builder.merge(orientation)?;
        }
        if let Some(output_color) = self.output_color {
            ticket_builder.merge(output_color)?;
        }
        if let Some(job_duplex) = self.job_duplex {
            ticket_builder.merge(job_duplex)?;
        }
        if let Some(resolution) = self.resolution {
            ticket_builder.merge(resolution)?;
        }
        Ok(ticket_builder.build()?)
    }
}
