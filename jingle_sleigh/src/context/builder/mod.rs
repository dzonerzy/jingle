use crate::context::builder::language_def::{parse_ldef, LanguageDefinition};
use crate::context::builder::processor_spec::parse_pspec;
use crate::context::SleighContext;
use crate::error::JingleSleighError;
use crate::error::JingleSleighError::{InvalidLanguageId, LanguageSpecRead};
use rust_embed::RustEmbed;
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{event, instrument, Level};
use vfs::FileSystem;

pub(crate) mod language_def;
pub(crate) mod processor_spec;

#[derive(Debug, Default, Clone)]
pub struct SleighContextBuilder<T: FileSystem> {
    defs: Vec<(LanguageDefinition, PathBuf)>,
    fs: T,
}

impl<T: FileSystem> SleighContextBuilder<T> {
    pub fn get_language_ids(&self) -> Vec<&str> {
        self.defs.iter().map(|(l, _)| l.id.as_str()).collect()
    }

    fn get_language(&self, id: &str) -> Option<&(LanguageDefinition, PathBuf)> {
        self.defs.iter().find(|(p, _)| p.id.eq(id))
    }
    #[instrument(skip_all, fields(%id))]
    pub fn build(&self, id: &str) -> Result<SleighContext, JingleSleighError> {
        let (lang, path) = self.get_language(id).ok_or(InvalidLanguageId)?;
        // LanguageDef contains the sla file path which should be concatenated with the path of the ldefs file to get the full path
        // assume path of ldefs file is /x86/data/languages/x86.ldefs and sla file is x86.sla
        // then the full path of the sla file is /x86/data/languages/x86.sla
        let sla_path = path
            .parent()
            .ok_or(JingleSleighError::PathError)?
            .join(&lang.sla_file);

        // Open and read the .sla file using the custom FileSystem trait
        let mut file = self
            .fs
            .open_file(&sla_path.to_string_lossy())
            .map_err(|_| JingleSleighError::FileReadError)?;

        let mut sla_bytes = Vec::new();
        file.read_to_end(&mut sla_bytes)
            .map_err(|_| JingleSleighError::FileReadError)?;

        let mut context = SleighContext::new(&sla_bytes, lang.id.clone())?;
        event!(Level::INFO, "Created sleigh context");
        // Construct the full path to the processor specification file (.pspec)
        let pspec_path = path
            .parent()
            .ok_or(JingleSleighError::PathError)?
            .join(&lang.processor_spec);

        // Parse the .pspec file using the custom FileSystem trait
        let pspec = parse_pspec(&self.fs, &pspec_path)?;

        if let Some(ctx_sets) = pspec.context_data.and_then(|d| d.context_set) {
            for set in ctx_sets.sets {
                // todo: gross hack
                if set.value.starts_with("0x") {
                    context.set_initial_context(
                        &set.name,
                        u32::from_str_radix(&set.value[2..], 16).unwrap(),
                    )?;
                } else {
                    context.set_initial_context(&set.name, set.value.parse::<u32>().unwrap())?;
                }
            }
        }
        Ok(context)
    }

    pub fn load_assets(fs: T) -> Result<Self, JingleSleighError> {
        let ldef: Vec<(LanguageDefinition, PathBuf)> = Self::_load_assets(&fs)?;
        Ok(SleighContextBuilder { defs: ldef, fs })
    }

    fn _load_assets(fs: &T) -> Result<Vec<(LanguageDefinition, PathBuf)>, JingleSleighError> {
        let mut defs = vec![];

        // Iterate over the entries in the root directory
        for entry in fs.read_dir(".")? {
            let path = PathBuf::from(".").join(&entry); // Construct the full path
            println!("Checking path: {:?}", path);
            // Check if the file has a ".ldefs" extension
            if let Some(extension) = path.extension() {
                if extension == "ldefs" {
                    // Parse the .ldefs file using the custom FileSystem
                    let parsed_defs = parse_ldef(fs, &path)?;

                    // Push each LanguageDefinition along with its path
                    for def in parsed_defs {
                        defs.push((def, path.clone()));
                    }
                }
            }
        }

        Ok(defs)
    }
}

fn find_ldef(path: &Path) -> Result<PathBuf, JingleSleighError> {
    for entry in (fs::read_dir(path).map_err(|_| LanguageSpecRead)?).flatten() {
        if let Some(e) = entry.path().extension() {
            if e == "ldefs" {
                return Ok(entry.path().clone());
            }
        }
    }
    Err(LanguageSpecRead)
}

#[cfg(test)]
mod tests {
    use crate::context::builder::processor_spec::parse_pspec;
    use crate::context::builder::{parse_ldef, SleighContextBuilder};

    use crate::tests::SLEIGH_ARCH;
    use rust_embed::RustEmbed;
    use std::path::Path;

    use vfs::EmbeddedFS;

    #[derive(RustEmbed, Debug)]
    #[folder = "assets"]
    struct AssetsEmbed;

    fn get_test_fs() -> EmbeddedFS<AssetsEmbed> {
        EmbeddedFS::new()
    }

    #[test]
    fn test_parse_ldef() {
        let fs = get_test_fs();
        parse_ldef(&fs, Path::new("processors/x86/data/languages/x86.ldefs")).unwrap();
    }

    #[test]
    fn test_parse_pspec() {
        let fs = get_test_fs();
        parse_pspec(&fs, Path::new("processors/x86/data/languages/x86.pspec")).unwrap();
    }

    /*#[test]
    fn test_parse_language_folder() {
        SleighContextBuilder::load_folder(Path::new(
            "ghidra/Ghidra/Processors/x86/data/languages/",
        ))
        .unwrap();
        SleighContextBuilder::load_folder(Path::new("ghidra/Ghidra/Processors/x86/data/languages"))
            .unwrap();
    }

    #[test]
    fn test_parse_language_ghidra() {
        let _builder = SleighContextBuilder::load_ghidra_installation(Path::new("ghidra")).unwrap();
    }

    #[test]
    fn test_get_language() {
        let langs = SleighContextBuilder::load_folder(Path::new(
            "ghidra/Ghidra/Processors/x86/data/languages/",
        ))
        .unwrap();
        assert!(langs.get_language("sdf").is_none());
        assert!(langs.get_language(SLEIGH_ARCH).is_some());
    }*/
}
