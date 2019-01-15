// Copyright 2017-2018 Kisio Digital and/or its affiliates.
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see
// <http://www.gnu.org/licenses/>.

//! See function apply_rules

use crate::collection::{CollectionWithId, Id};
use crate::model::Collections;
use crate::objects::Codes;
use crate::utils::{Report, ReportType};
use crate::Result;
use csv;
use failure::ResultExt;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
enum ObjectType {
    Line,
    Route,
    StopPoint,
    StopArea,
}
impl ObjectType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ObjectType::Line => "line",
            ObjectType::Route => "route",
            ObjectType::StopPoint => "stop_point",
            ObjectType::StopArea => "stop_area",
        }
    }
}

#[derive(Deserialize, Debug, Ord, Eq, PartialOrd, PartialEq, Clone)]
struct ComplementaryCode {
    object_type: ObjectType,
    object_id: String,
    object_system: String,
    object_code: String,
}

fn read_complementary_code_rules_files<P: AsRef<Path>>(
    rule_files: Vec<P>,
    report: &mut Report,
) -> Result<Vec<ComplementaryCode>> {
    info!("Reading complementary code rules.");
    let mut codes = BTreeSet::new();
    for rule_path in rule_files {
        let path = rule_path.as_ref();
        let mut rdr = csv::Reader::from_path(&path).with_context(ctx_from_path!(path))?;
        for c in rdr.deserialize() {
            let c: ComplementaryCode = match c {
                Ok(val) => val,
                Err(e) => {
                    report.add_warning(
                        format!("Error reading {:?}: {}", path.file_name().unwrap(), e),
                        ReportType::ComplementaryCodeRulesRead,
                    );
                    continue;
                }
            };
            codes.insert(c);
        }
    }
    Ok(codes.into_iter().collect())
}

fn insert_code<T>(
    collection: &mut CollectionWithId<T>,
    code: ComplementaryCode,
    report: &mut Report,
) where
    T: Codes + Id<T>,
{
    let idx = match collection.get_idx(&code.object_id) {
        Some(idx) => idx,
        None => {
            report.add_warning(
                format!(
                    "Error inserting code: object_codes.txt: object={},  object_id={} not found",
                    code.object_type.as_str(),
                    code.object_id
                ),
                ReportType::ComplementaryObjectNotFound,
            );
            return;
        }
    };

    collection
        .index_mut(idx)
        .codes_mut()
        .insert((code.object_system, code.object_code));
}

/// Applying rules
///
/// `complementary_code_rules_files` Csv files containing codes to add for certain objects
pub fn apply_rules(
    collections: &mut Collections,
    complementary_code_rules_files: Vec<PathBuf>,
    report_path: PathBuf,
) -> Result<()> {
    info!("Applying rules...");
    let mut report = Report::default();
    let codes = read_complementary_code_rules_files(complementary_code_rules_files, &mut report)?;

    for code in codes {
        match code.object_type {
            ObjectType::Line => insert_code(&mut collections.lines, code, &mut report),
            ObjectType::Route => insert_code(&mut collections.routes, code, &mut report),
            ObjectType::StopPoint => insert_code(&mut collections.stop_points, code, &mut report),
            ObjectType::StopArea => insert_code(&mut collections.stop_areas, code, &mut report),
        }
    }

    let serialized_report = serde_json::to_string_pretty(&report)?;
    fs::write(report_path, serialized_report)?;
    Ok(())
}

#[cfg(test)]
mod tests {
#[test]
fn bob() {
    // test to check if it's possible to import the builder
    let model = crate::model_builder::ModelBuilder::default()
        .vj("toto", |vj_builder| {
            vj_builder
                .st("A", "10:00:00", "10:01:00")
                .st("B", "11:00:00", "11:01:00");
        })
        .vj("tata", |vj_builder| {
            vj_builder
                .st("C", "10:00:00", "10:01:00")
                .st("D", "11:00:00", "11:01:00");
        })
        .build();

    assert_eq!(
        model.get_corresponding_from_idx(model.vehicle_journeys.get_idx("toto").unwrap()),
        ["A", "B"]
            .into_iter()
            .map(|s| model.stop_points.get_idx(s).unwrap())
            .collect()
    );
    //assert!(false);
}
}
