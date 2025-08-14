use loro::LoroDoc;

use crate::models::doc_update::DocUpdatePayload;

pub type StateDoc = LoroDoc;

impl StateDoc {
    pub fn new(updates: Vec<DocUpdatePayload>) -> Self {
        let doc = LoroDoc::new();
        match doc.import_batch(&updates) {
            Ok(_) => event!(Level::DEBUG, "Building document success."),
            Err(err) => event!(
                Level::ERROR,
                "Error building the document from stored updates.\n{}",
                err
            ),
        };
        doc.subscribe();
        doc
    }
}