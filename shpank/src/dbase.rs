use std::{ffi::CStr, io};

use crate::parse::{Error, Parser, Result};

#[derive(Debug)]
pub struct DbaseFile {
    pub header: DbaseHeader,
    pub records: Vec<DbaseRecord>,
}

#[derive(Debug)]
pub struct DbaseRecord {
    pub entries: Vec<String>,
}

// https://en.wikipedia.org/wiki/.dbf#File_format_of_Level_5_DOS_dBASE
#[derive(Debug)]
pub struct DbaseHeader {
    pub flags: u8,

    /// Last update: Years since 1900
    pub yy: u8,
    /// Last update: Month
    pub mm: u8,
    /// Last update: Days
    pub dd: u8,

    /// How many records in total
    // Stored as u32
    pub num_records: usize,

    /// How many bytes in the header in total
    // Stored as u16
    pub header_bytes: usize,

    /// How many bytes in a record in total
    // Stored as u16
    pub record_bytes: usize,

    pub fields: Vec<FieldDescriptor>,
}

impl DbaseHeader {
    /// Compares the provided field name (in trimmed lowercase ASCII)
    /// to the declared fields (in trimmed lowercase ASCII).
    /// If a match is found returns the index.
    pub fn index_of(&self, field_name: &str) -> Option<usize> {
        self.fields.iter().position(|field| {
            field.name.to_ascii_lowercase().trim() == field_name.to_ascii_lowercase().trim()
        })
    }
}

impl DbaseHeader {
    /// Total bytes in a .dbf file:
    /// - Header bytes
    /// - Record bytes
    /// - EOF byte
    pub fn dbase_num_bytes_total(&self) -> usize {
        // File ends with an EOF
        self.dbase_num_bytes_header_and_records() + 1
    }

    pub fn dbase_num_bytes_header_and_records(&self) -> usize {
        self.header_bytes + (self.num_records * self.record_bytes)
    }
}

// https://en.wikipedia.org/wiki/.dbf#Database_records
#[derive(Debug)]
#[repr(u8)]
pub enum FieldType {
    Character = 'C' as u8,
    Date = 'D' as u8,
    FloatingPoint = 'F' as u8,
    Logical = 'L' as u8,
    Memo = 'M' as u8,
    Numeric = 'N' as u8,
}

impl TryFrom<char> for FieldType {
    type Error = Error;

    fn try_from(value: char) -> Result<Self> {
        let value: u8 = value.try_into()?;
        Ok(match value {
            v if v == FieldType::Character as u8 => FieldType::Character,
            v if v == FieldType::Date as u8 => FieldType::Date,
            v if v == FieldType::FloatingPoint as u8 => FieldType::FloatingPoint,
            v if v == FieldType::Logical as u8 => FieldType::Logical,
            v if v == FieldType::Memo as u8 => FieldType::Memo,
            v if v == FieldType::Numeric as u8 => FieldType::Numeric,

            _ => {
                return Err(Error::UnexpectedData(format!(
                    "The char `{value}` does not correspond to a field type"
                )))
            }
        })
    }
}

// https://en.wikipedia.org/wiki/.dbf#Field_descriptor_array
#[derive(Debug)]
pub struct FieldDescriptor {
    pub name: String,
    pub type_: FieldType,

    /// Number of bytes in this field
    pub field_length: usize,
    // TODO: There is more but lets add if they occur in .dbf files
}

impl<R> Parser<R>
where
    R: io::Read,
{
    pub fn parse_dbase_header(&mut self) -> Result<DbaseHeader> {
        assert_eq!(self.num_bytes_read(), 0);

        let flags = self.parse_u8()?;
        let (yy, mm, dd) = (self.parse_u8()?, self.parse_u8()?, self.parse_u8()?);
        let num_records = self.parse_u32_le()? as usize;
        let header_bytes = self.parse_u16_le()? as usize;
        let record_bytes = self.parse_u16_le()? as usize;

        // Misc we don't care about
        self.read_exact(&mut [0; 20])?;

        // 1 byte extra for terminator 0x0D after field descriptors
        let non_field_header_bytes = self.num_bytes_read() + 1;

        let field_descr_bytes = header_bytes
            .checked_sub(non_field_header_bytes)
            .ok_or_else(|| Error::UnexpectedData("Too few bytes in header".into()))?;

        assert!(field_descr_bytes % 32 == 0);

        let num_fields = field_descr_bytes / 32;

        let mut fields = vec![];
        for _ in 0..num_fields {
            fields.push(self.parse_dbase_field_descriptor()?);
        }

        let terminator = self.parse_u8()?;
        assert_eq!(terminator, 0x0D, "expected terminator char 0x0D");

        Ok(DbaseHeader {
            flags,
            yy,
            mm,
            dd,
            num_records,
            header_bytes,
            record_bytes,
            fields,
        })
    }

    pub fn parse_dbase_field_type(&mut self) -> Result<FieldType> {
        self.parse_ascii()?.try_into()
    }

    pub fn parse_dbase_field_descriptor(&mut self) -> Result<FieldDescriptor> {
        let num_read_start = self.num_bytes_read();

        let mut name_bytes = vec![0; 11];
        self.read_exact(&mut name_bytes)?;
        let name = CStr::from_bytes_until_nul(&name_bytes[..])?
            .to_str()
            .map_err(|e| Error::UnexpectedData(format!("Utf8: {e:?}")))?
            .to_string();

        let type_ = self.parse_dbase_field_type()?;

        // 4 reserved bytes
        self.read_exact(&mut [0; 4])?;

        let field_length = self.parse_u8()? as usize;

        // 15 bytes we don't care about
        self.read_exact(&mut [0; 15])?;

        let num_read_end = self.num_bytes_read();

        let num_read = num_read_end
            .checked_sub(num_read_start)
            .ok_or_else(|| Error::UnexpectedData("Did not read 32 bytes".into()))?;
        // Expected size of field descr
        assert_eq!(num_read, 32);

        Ok(FieldDescriptor {
            name,
            type_,
            field_length,
        })
    }

    pub fn parse_dbase_record(&mut self, header: &DbaseHeader) -> Result<DbaseRecord> {
        // https://en.wikipedia.org/wiki/.dbf#Database_records
        // > Each record begins with a 1-byte "deletion" flag. The byte's value is a space (0x20), if the record is active, or an asterisk (0x2A), if the record is deleted.
        let flag = self.parse_u8()?;
        assert_eq!(
            0x20,
            flag,
            "only active records handled (byte pos {})",
            self.num_bytes_read()
        );

        let mut entries = vec![];
        for FieldDescriptor {
            name: _,
            type_,
            field_length,
        } in &header.fields
        {
            let mut buf = vec![0; *field_length];
            self.read_exact(&mut buf)?;

            let entry = match type_ {
                FieldType::Date => unimplemented!(),
                FieldType::FloatingPoint => unimplemented!(),
                FieldType::Logical => unimplemented!(),
                FieldType::Memo => unimplemented!(),
                FieldType::Character => std::str::from_utf8(&buf)?.trim_end().to_string(),
                FieldType::Numeric => std::str::from_utf8(&buf)?.trim_end().to_string(),
            };

            entries.push(entry.trim_end().to_string());
        }

        Ok(DbaseRecord { entries })
    }

    pub(crate) fn impl_parse_dbase_file(mut self) -> Result<DbaseFile> {
        let header = self.parse_dbase_header()?;

        let mut records = vec![];

        let goal = header.dbase_num_bytes_header_and_records();

        while self.num_bytes_read() < goal {
            records.push(self.parse_dbase_record(&header)?);
        }

        assert_eq!(self.num_bytes_read(), goal);

        Ok(DbaseFile { header, records })
    }
}
