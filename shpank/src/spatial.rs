use crate::{
    dbase::DbaseFile,
    parse::{self, Error, Result},
    shape::ShpFile,
};

pub struct Spatial {
    shp: ShpFile,
    dbf: DbaseFile,
}

impl Spatial {
    pub fn new(shp: &str, dbf: &str) -> Result<Self> {
        let shp = parse::Parser::parse_shp_file(shp)?;
        let dbf = parse::Parser::parse_dbf_file(dbf)?;

        let shp_num = shp.records.len();
        let dbf_num = dbf.records.len();
        if shp_num != dbf_num {
            return Err(Error::UnexpectedData(format!(
                "Shapefile # records not equal to dBASE: {shp_num} vs {dbf_num}"
            )));
        }

        Ok(Self { shp, dbf })
    }
}

impl Spatial {
    pub fn num_records(&self) -> usize {
        self.shp.records.len()
    }
}
