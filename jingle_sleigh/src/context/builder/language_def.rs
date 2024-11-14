use crate::error::JingleSleighError;
use crate::error::JingleSleighError::LanguageSpecRead;
use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use vfs::FileSystem;

#[derive(Clone, Debug, Deserialize)]
pub enum SleighEndian {
    #[serde(rename = "little")]
    Little,
    #[serde(rename = "big")]
    Big,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
pub struct Compiler {
    pub name: String,
    pub spec: String,
    pub id: String,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
pub struct ExternalName {
    pub tool: String,
    pub name: String,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
pub struct LanguageDefinition {
    pub processor: String,
    pub endian: SleighEndian,
    pub variant: String,
    pub version: String,
    #[serde(rename = "slafile")]
    pub sla_file: PathBuf,
    #[serde(rename = "processorspec")]
    pub processor_spec: PathBuf,
    #[serde(rename = "manualindexfile")]
    pub manual_index_file: Option<PathBuf>,
    pub id: String,
    pub description: String,
    pub compiler: Vec<Compiler>,
    pub external_name: Option<Vec<ExternalName>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "language_definitions")]
struct LanguageDefinitions {
    #[serde(rename = "$value")]
    pub language_definitions: Vec<LanguageDefinition>,
}

pub(super) fn parse_ldef(
    fs: &dyn FileSystem,
    path: &Path,
) -> Result<Vec<LanguageDefinition>, JingleSleighError> {
    // Convert the Path to a string for compatibility with the FileSystem trait
    let path_str = path.to_string_lossy();

    // Use the FileSystem trait to open the file
    let mut file = fs
        .open_file(&path_str)
        .map_err(|_| JingleSleighError::LanguageSpecRead)?;

    // Read the contents of the file into a buffer
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)
        .map_err(|_| JingleSleighError::LanguageSpecRead)?;

    // Deserialize XML content from the buffer
    let def: LanguageDefinitions = serde_xml_rs::from_reader(file_content.as_slice())?;
    Ok(def.language_definitions)
}

#[cfg(test)]
mod tests {
    use crate::context::builder::language_def::LanguageDefinitions;
    use serde_xml_rs::from_str;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test() {
        let mut file = File::open("ghidra/Ghidra/Processors/x86/data/languages/x86.ldefs").unwrap();
        let mut data: String = String::new();
        file.read_to_string(&mut data).unwrap();
        let _ldef: LanguageDefinitions = from_str(&data).unwrap();
    }
}
