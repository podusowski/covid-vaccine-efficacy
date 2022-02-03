use serde::{Deserialize, Deserializer};

use crate::YearWeek;

fn deserialize_year_week<'a, D>(deserializer: D) -> Result<YearWeek, D::Error>
where
    D: Deserializer<'a>,
{
    let text: &str = Deserialize::deserialize(deserializer)?;
    let re = regex::Regex::new(r"(\d{4})-W(\d{2})").unwrap();
    let captures = re.captures(text).unwrap();
    Ok(YearWeek((
        captures.get(1).unwrap().as_str().parse().unwrap(),
        captures.get(2).unwrap().as_str().parse().unwrap(),
    )))
}

fn deserialize_age_group<'a, D>(deserializer: D) -> Result<(usize, usize), D::Error>
where
    D: Deserializer<'a>,
{
    let text: &str = Deserialize::deserialize(deserializer)?;

    match text {
        "Age0_4" => Ok((0, 4)),
        "Age5_9" => Ok((5, 9)),
        "Age10_14" => Ok((10, 14)),
        "Age15_17" => Ok((15, 17)),
        "Age18_24" => Ok((18, 24)),
        "Age25_49" => Ok((25, 49)),
        "Age50_59" => Ok((50, 59)),
        "Age60_69" => Ok((60, 69)),
        "Age70_79" => Ok((70, 79)),
        "Age80+" => Ok((80, 120)),
        "ALL" | "AgeUNK" | "HCW" => Err(serde::de::Error::custom(format!(
            "don't care about this one '{}'",
            text
        ))),
        _ => panic!("{}", text),
    }
}

#[derive(Debug, Deserialize)]
struct VaccinationEcdcRow {
    #[serde(alias = "YearWeekISO", deserialize_with = "deserialize_year_week")]
    year_week: YearWeek,
    #[serde(alias = "ReportingCountry")]
    country: String,
    #[serde(alias = "Region")]
    region: String,
    #[serde(alias = "TargetGroup", deserialize_with = "deserialize_age_group")]
    age_group: (usize, usize),
    #[serde(alias = "FirstDose")]
    first_dose: usize,
    #[serde(alias = "SecondDose")]
    second_dose: usize,
    #[serde(alias = "DoseAdditional1")]
    third_dose: usize,
}

fn read_vaccinations() -> Vec<VaccinationEcdcRow> {
    let file = std::fs::File::open("data/vaccines-pl.csv").unwrap();
    csv::ReaderBuilder::new()
        .from_reader(file)
        .deserialize::<VaccinationEcdcRow>()
        .filter_map(Result::ok)
        .filter(|r| r.country == "PL" && r.region == "PL")
        .collect()
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct VaccinatedPeople {
    pub at_least_one_dose: usize,
    pub at_least_two_doses: usize,
    pub at_least_three_doses: usize,
    pub one_dose: usize,
    pub two_doses: usize,
    pub three_doses: usize,
}

impl VaccinatedPeople {
    fn update(self, rhs: &VaccinationEcdcRow) -> Self {
        Self {
            at_least_one_dose: self.at_least_one_dose + rhs.first_dose,
            at_least_two_doses: self.at_least_two_doses + rhs.second_dose,
            at_least_three_doses: self.at_least_three_doses + rhs.third_dose,
            // Doing checked subtraction since there are some discrepancies in the ECDC data,
            // showing couple people as vaccinated with booster before getting second dose.
            one_dose: (self.one_dose + rhs.first_dose)
                .checked_sub(rhs.second_dose)
                .unwrap_or_default(),
            two_doses: (self.two_doses + rhs.second_dose)
                .checked_sub(rhs.third_dose)
                .unwrap_or_default(),
            three_doses: self.three_doses + rhs.third_dose,
        }
    }
}

#[derive(Default)]
pub(crate) struct VaccinationData {
    rows: Vec<VaccinationEcdcRow>,
}

impl VaccinationData {
    pub fn sum(&self, age_group: (usize, usize), week: YearWeek) -> VaccinatedPeople {
        self.rows
            .iter()
            .filter(|row| row.year_week <= week && row.age_group == age_group)
            .fold(VaccinatedPeople::default(), VaccinatedPeople::update)
    }

    pub fn new() -> Self {
        Self {
            rows: read_vaccinations(),
        }
    }
}
