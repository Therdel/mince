use std::{collections::HashMap, ops::Range};

use anyhow::{anyhow, Result, Context};

use crate::ModuleName;

pub struct MemVars {
    pub on_ground_op_dec:   *mut [u8],
    pub on_ground_op_inc:   *mut [u8],
    pub on_ground:          *mut u8,
    pub do_jump:            *mut u8,
}

impl MemVars {
    pub fn read() -> Result<Self> {
        let signatures = Signatures::new()?;
        let signature_areas = SignatureAreas::new(&signatures)?;

        let get_signature_area = |signature_area_name: &str| {
            signature_areas.areas.get(signature_area_name)
                .context(anyhow!("Signature area {signature_area_name} not found"))
        };

        let read_mut_pointer = |signature_area_name: &str| -> Result<*mut u8> {
            let signature_area = get_signature_area(signature_area_name)?;
            let scan_result = SignatureScanner::scan_signature(signature_area, &signatures)?;
            let scan_result: &[u8] = unsafe { &*scan_result};
            let pointer = usize::from_le_bytes(scan_result.try_into()?);
            Ok(pointer as *mut u8)
        };

        let read_mut_slice = |signature_area_name: &str| -> Result<*mut [u8]> {
            let signature_area = get_signature_area(signature_area_name)?;
            let slice = SignatureScanner::scan_signature(signature_area, &signatures)?;
            Ok(slice as *mut[u8])
        };
        
        let mem_vars = MemVars {
            on_ground_op_dec: read_mut_slice("on_ground_op_dec")?,
            on_ground_op_inc: read_mut_slice("on_ground_op_inc")?,
            on_ground: read_mut_pointer("on_ground")?,
            do_jump: read_mut_pointer("do_jump")?,
        };
        Ok(mem_vars)
    }
}
    
pub struct SignatureAreas {
    areas: HashMap<String, SignatureArea>,
}

impl SignatureAreas {
    #[cfg(target_os="windows")]
    pub fn new(signatures: &Signatures) -> Result<Self> {
        let signature_areas = HashMap::from([
            ("on_ground_op_dec".into(), SignatureArea::new("on_ground_dec".into(), SignatureAreaType::Range(0..6))),
            ("on_ground_op_inc".into(), SignatureArea::new("on_ground_inc".into(), SignatureAreaType::Range(0..6))),
            ("on_ground".into(),        SignatureArea::new("on_ground_dec".into(), SignatureAreaType::CaptureGroupIndex(1))),
            ("do_jump".into(),          SignatureArea::new("do_jump_update".into(), SignatureAreaType::CaptureGroupIndex(1))),
        ]);
        let result = Self { areas: signature_areas };
        result.check_referenced_signatures(signatures)?;
        Ok(result)
    }

    fn check_referenced_signatures(&self, signatures: &Signatures) -> Result<()> {
        for (area_description, area) in &self.areas {
            if !signatures.signatures.contains_key(&area.signature_name) {
                return Err(anyhow!("Signature {} referenced by signature area {} not found in signatures", area.signature_name, area_description))
            }
        }
        Ok(())
    }
}

pub struct Signatures {
    signatures: HashMap<String, Signature>,
}

impl Signatures {
    pub fn new() -> Result<Self> {
        #[cfg(target_os="windows")]
        let signatures = HashMap::from([
            ("on_ground_dec".into(),       Signature::new(ModuleName::Client, "FF 0D ?? ?? ?? ?? E8 ?? ?? ?? ?? 5E 5D")?),
            ("on_ground_inc".into(),       Signature::new(ModuleName::Client, "FF 05 ?? ?? ?? ?? 85 DB 74 0D 8B 13")?),
            ("do_jump_update".into(),      Signature::new(ModuleName::Client, "89 0D ?? ?? ?? ?? 8B 0D ?? ?? ?? ?? F6 C1 03 74 03 83 CE 08")?),
            ]);
        Ok(Signatures { signatures })
    }
}

pub enum SignatureAreaType {
    /// Index of the [capture group](https://docs.rs/regex/1.7.1/regex/index.html#example-iterating-over-capture-groups)
    CaptureGroupIndex(usize),
    /// Range in the signature bytes
    Range(Range<usize>)
}

pub struct SignatureArea {
    signature_name: String,
    /// region of the signature pattern this signature is "interested in".
    area: SignatureAreaType,
}

impl SignatureArea {
    fn new(signature_name: String, area: SignatureAreaType) -> Self {
        Self { signature_name, area }
    }
}

pub struct Signature {
    module: ModuleName,
    // signature: String,
    regex: regex::bytes::Regex
}

impl Signature {
    fn new(module: ModuleName, signature: &str) -> Result<Self> {
        // (?s-u)    | flags: -u = match invalid unicode, s: . matches all bytes (only in bytes mode)
        // \xFF      | match literal byte
        // (exp)     | numbered capture group
        // .{n}      | match any byte n times
        // ^         | match beginning of input (force include beginning in match)
        // $         | match end of input (force to include end in match)
        // source: https://docs.rs/regex/latest/regex/#syntax
        // source: https://docs.rs/regex/latest/regex/bytes/index.html#syntax

        // space-seperated pairs of either ?? or hex bytes, case-insensitive, match whole input
        let regex_signature_text_format: regex::Regex = "(?i:^(([[:xdigit:]]|\\?){2}[[:space:]])*([[:xdigit:]]|\\?){2}$)".parse()?;
        if !regex_signature_text_format.is_match(signature) {
            return Err(anyhow!("Signature has wrong format: '{signature}'"));
        }

        let regex_pattern_begin = "(?s-u:";
        let regex_pattern_end = ")";
        let (begin_group, end_group) = ("(", ")");
        let wildcard_byte = ".";
        
        let mut regex_signature = regex_pattern_begin.to_string();

        let is_wildcard = |byte| byte == "??";
        let mut last_inside_wildcard = false;
        for elem in signature.to_uppercase().split_whitespace() {
            if is_wildcard(elem) {
                if !last_inside_wildcard {
                    regex_signature += begin_group;
                    last_inside_wildcard = true;
                }
                regex_signature += wildcard_byte;
            } else {
                if last_inside_wildcard {
                    regex_signature += end_group;
                    last_inside_wildcard = false;
                }
                regex_signature += r"\x";
                regex_signature += elem;
            }
        }
        if last_inside_wildcard {
            regex_signature += end_group;
        }
        regex_signature += regex_pattern_end;

        let regex = regex_signature.parse()?;
        Ok(Signature { module, /* signature: signature.to_string(),*/ regex })
    }
}

struct SignatureScanner;
impl SignatureScanner {
    fn scan_signature(signature_area: &SignatureArea, signatures: &Signatures) -> Result<*const [u8]> {
        let signature_name = &signature_area.signature_name;
        let Some(signature) = signatures.signatures.get(signature_name) else {
            return Err(anyhow!("Signature {} not found", signature_name))
        };
        
        // get module region
        let module_mapping = module_maps::find_module(|mapping| mapping.file_name == signature.module.file_name())?;
        let module_mapping = module_mapping
            .context(anyhow!("Module {} not found in memory maps", signature.module.file_name()))?;
        let region = unsafe { &*module_mapping.memory };

        // search module for signature using regex
        let matches: Vec<_> = signature.regex.captures_iter(region).collect();
        if matches.len() != 1 {
            return Err(anyhow!("Signature {signature_name} had {} matches", matches.len()))
        }
        let regex_match = matches.first().unwrap();
    
        // extract match area of interest
        let match_area_of_interest = match &signature_area.area {
            SignatureAreaType::CaptureGroupIndex(capture_index) => {
                regex_match.get(*capture_index)
                    .context(anyhow!("Signature area capture group index {capture_index} of Signature {} doesn't exist", signature_area.signature_name))?
                    .as_bytes()
            },
            SignatureAreaType::Range(range) => {
                let whole_match = 0;
                let bytes = regex_match.get(0)
                    .context(anyhow!("Signature area capture group index {whole_match} of Signature {} doesn't exist", signature_area.signature_name))?
                    .as_bytes();
                bytes.get(range.clone())
                    .context(anyhow!("Signature area wasn't indexable with range {range:?}"))?
            },
        };

        Ok(match_area_of_interest)
    }
}
