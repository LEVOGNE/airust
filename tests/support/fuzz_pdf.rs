use airust::{PdfImportLimits, PdfLoader};

pub struct PdfCampaign;

impl PdfCampaign {
    pub fn exercise(data: &[u8]) {
        let dir = tempfile::tempdir().expect("pdf directory");
        let path = dir.path().join("input.pdf");
        std::fs::write(&path, data).expect("pdf bytes");
        let limits = PdfImportLimits::new(262_144, 32, 1_048_576, 256).expect("pdf limits");
        let loader = PdfLoader::new().with_limits(limits);
        let _ = loader.extract_pages_from_pdf(&path);
        let _ = loader.preview_pages_from_pdf(&path, 2);
        let _ = loader.pdf_to_training_examples(&path);
    }
}
