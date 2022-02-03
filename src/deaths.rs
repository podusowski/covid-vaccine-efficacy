use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Deserializer};

use crate::{AgeGroup, YearWeek};

fn deserialize_date<'a, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'a>,
{
    let text: &str = Deserialize::deserialize(deserializer)?;
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .map_err(|_| serde::de::Error::custom(format!("bad date: '{}'", text)))
}

fn deserialize_age<'a, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'a>,
{
    let text: &str = Deserialize::deserialize(deserializer)?;
    let age: f32 = text
        .parse()
        .map_err(|_| serde::de::Error::custom(format!("bad age: '{}'", text)))?;
    Ok(age.round() as usize)
}

#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
pub(crate) enum VaccinationStatus {
    #[serde(rename = "")]
    Unvaccinated,
    #[serde(rename = "jedna_dawka")]
    OneDose,
    #[serde(alias = "dwie_dawki", alias = "pelna_dawka")]
    TwoDoses,
    #[serde(alias = "uzupełniajšca", alias = "przypominajaca")]
    ThreeDoses,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub(crate) struct CovidDeath {
    #[serde(alias = "data_rap_zgonu", deserialize_with = "deserialize_date")]
    date: NaiveDate,
    #[serde(alias = "wiek", deserialize_with = "deserialize_age")]
    pub age: usize,
    #[serde(alias = "dawka_ost")]
    pub vaccination_status: VaccinationStatus,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub(crate) struct Cases {
    #[serde(alias = "data_rap_zakazenia", deserialize_with = "deserialize_date")]
    date: NaiveDate,
    #[serde(alias = "wiek", deserialize_with = "deserialize_age")]
    pub age: usize,
    #[serde(alias = "dawka_ost")]
    pub vaccination_status: VaccinationStatus,
    #[serde(alias = "liczba_zaraportowanych_zakazonych")]
    pub count: usize,
}

pub(crate) struct DeathsData {
    pub total_deaths: usize,
    deaths: Vec<CovidDeath>,
}

impl DeathsData {
    pub fn new() -> Self {
        let file = std::fs::File::open("data/ewp_dsh_zgony_po_szczep_20211214.csv").unwrap();
        let transcoded = encoding_rs_io::DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding_rs::ISO_8859_2))
            .build(file);

        let deaths: Vec<_> = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(transcoded)
            .deserialize::<CovidDeath>()
            .filter_map(|r| match r {
                Ok(r) => Some(r),
                Err(e) => {
                    println!("dropping record: {}", e);
                    None
                }
            })
            .collect();

        Self {
            total_deaths: deaths.len(),
            deaths,
        }
    }

    pub fn by_vaccination_status(
        &self,
        week: YearWeek,
        age_group: AgeGroup,
        vaccination_status: VaccinationStatus,
    ) -> usize {
        self.deaths
            .iter()
            .filter(|death| {
                YearWeek::from(death.date.iso_week()) == week
                    && age_group.includes(death.age)
                    && death.vaccination_status == vaccination_status
            })
            .count()
    }
}

pub(crate) struct InfectionsData {
    pub cases: Vec<Cases>,
}

impl InfectionsData {
    pub fn new() -> Self {
        let file =
            std::fs::File::open("data/ewp_dsh_zakazenia_po_szczepieniu_202202010940.csv").unwrap();
        let transcoded = encoding_rs_io::DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding_rs::ISO_8859_2))
            .build(file);

        let cases: Vec<_> = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(transcoded)
            .deserialize::<Cases>()
            .filter_map(|r| match r {
                Ok(r) => Some(r),
                Err(e) => {
                    println!("dropping case record: {}", e);
                    None
                }
            })
            .collect();

        Self { cases }
    }

    pub fn by_vaccination_status(
        &self,
        week: YearWeek,
        age_group: AgeGroup,
        vaccination_status: VaccinationStatus,
    ) -> usize {
        self.cases
            .iter()
            .filter(|death| {
                YearWeek::from(death.date.iso_week()) == week
                    && age_group.includes(death.age)
                    && death.vaccination_status == vaccination_status
            })
            .map(|cases| cases.count)
            .sum()
    }
}
