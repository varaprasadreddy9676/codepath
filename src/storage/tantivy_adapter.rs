use tracing::{info, warn};
use tantivy::schema::*;
use tantivy::{Index, IndexWriter};

pub async fn initialize_lexical_index() {
    info!("Initializing Tantivy exact-lexical search adapter...");
    
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("identifier", TEXT | STORED);
    schema_builder.add_text_field("stacktrace", TEXT);
    
    let schema = schema_builder.build();
    
    // In Tantivy natively, create_in_ram directly returns an Index. 
    let index = Index::create_in_ram(schema.clone());
    info!("Tantivy exact-match RAM index generated successfully.");
    
    let mut index_writer: IndexWriter = match index.writer(50_000_000) {
        Ok(writer) => writer,
        Err(e) => {
            warn!("Failed to create Tantivy core writer: {}", e);
            return;
        }
    };
    
    // System guarantees safe commits across multi-threading environments
    if let Err(e) = index_writer.commit() {
        warn!("Tantivy commit operation failed internally: {}", e);
    }
}
