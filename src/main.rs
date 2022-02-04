use std::{collections::HashMap, fmt::Display, hash::Hash, ops::Add};

use chrono::{Datelike, IsoWeek, NaiveDate};
use demographics::age_distribution;
use statrs::statistics::Statistics;
use vaccination::VaccinatedPeople;

use crate::{
    deaths::{DeathsData, InfectionsData, VaccinationStatus},
    vaccination::VaccinationData,
};

mod deaths;
mod demographics;
mod plots;
mod tables;
mod vaccination;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct YearWeek((u32, u32));

impl From<IsoWeek> for YearWeek {
    fn from(week: IsoWeek) -> Self {
        YearWeek((week.year() as u32, week.week() as u32))
    }
}

impl Display for YearWeek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}W{:02}", self.0 .0, self.0 .1)
    }
}

fn weeks_of_2021() -> impl Iterator<Item = YearWeek> {
    NaiveDate::from_ymd(2021, 1, 1)
        .iter_weeks()
        .map(|week| YearWeek::from(week.iso_week()))
        .take(51)
}

fn weeks(max: YearWeek) -> impl Iterator<Item = YearWeek> {
    NaiveDate::from_ymd(2021, 1, 1)
        .iter_weeks()
        .map(|week| YearWeek::from(week.iso_week()))
        .take_while(move |week| week <= &max)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AgeGroup((usize, usize));

impl AgeGroup {
    const fn new(from: usize, to: usize) -> Self {
        Self((from, to))
    }

    fn includes(&self, age: usize) -> bool {
        age >= self.0 .0 && age <= self.0 .1
    }
}

impl Display for AgeGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.0 .0, self.0 .1)
    }
}

fn per_million(nominator: usize, denominator: usize) -> f64 {
    nominator as f64 * 1_000_000_f64 / denominator as f64
}

#[derive(Debug, Clone, Copy)]
struct DeathRate<T> {
    unvaccinated: T,
    two_doses: T,
    three_doses: T,
}

impl<T: Add<Output = T> + Copy> DeathRate<T> {
    fn total(&self) -> T {
        self.unvaccinated + self.two_doses + self.three_doses
    }
}

#[derive(Debug, Clone, Copy)]
struct WeeklyReport {
    vaccinated_people: VaccinatedPeople,
    unvaccinated_people: usize,
    absolute_cases: DeathRate<usize>,
    absolute_deaths: DeathRate<usize>,
    deaths_per_million: DeathRate<f64>,
    cases_per_million: DeathRate<f64>,
}

impl WeeklyReport {
    fn risk_ratio_of_two_doses(&self) -> f64 {
        self.deaths_per_million.two_doses / self.deaths_per_million.unvaccinated
    }

    fn risk_ratio_of_three_doses(&self) -> f64 {
        self.deaths_per_million.three_doses / self.deaths_per_million.unvaccinated
    }

    fn case_risk_ratio_of_two_doses(&self) -> f64 {
        self.cases_per_million.two_doses / self.cases_per_million.unvaccinated
    }

    fn case_risk_ratio_of_three_doses(&self) -> f64 {
        self.cases_per_million.three_doses / self.cases_per_million.unvaccinated
    }

    fn cfr_unvaccinated(&self) -> f64 {
        self.absolute_deaths.unvaccinated as f64 / self.absolute_cases.unvaccinated as f64
    }

    fn cfr_two_doses(&self) -> f64 {
        self.absolute_deaths.two_doses as f64 / self.absolute_cases.two_doses as f64
    }
}

// Deaths and demographics are grouped by year, ECDC vaccination data however is
// reported as age groups.
const AGE_GROUPS: &'static [AgeGroup] = &[
    AgeGroup::new(0, 4),
    AgeGroup::new(5, 9),
    AgeGroup::new(10, 14),
    AgeGroup::new(15, 17),
    AgeGroup::new(18, 24),
    AgeGroup::new(25, 49),
    AgeGroup::new(50, 59),
    AgeGroup::new(60, 69),
    AgeGroup::new(70, 79),
    // FIXME: Population is miscalculated for this group. Thank you GUS.
    //AgeGroup::new(80, 120),
];

/// This type should contain every source and calculated data needed for presentation.
struct WeeklyReports(Vec<(YearWeek, HashMap<AgeGroup, WeeklyReport>)>);

impl WeeklyReports {
    fn mean_risk_ratio_of_two_doses(&self) -> Vec<(YearWeek, f64)> {
        self.0
            .iter()
            .map(|(week, report)| {
                (
                    *week,
                    report
                        .values()
                        .map(WeeklyReport::risk_ratio_of_two_doses)
                        .filter(|rr| rr.is_finite())
                        .mean(),
                )
            })
            .collect()
    }

    fn mean_risk_ratio_of_three_doses(&self) -> Vec<(YearWeek, f64)> {
        self.0
            .iter()
            .map(|(week, report)| {
                (
                    *week,
                    report
                        .values()
                        .map(WeeklyReport::risk_ratio_of_three_doses)
                        .filter(|rr| rr.is_finite())
                        .mean(),
                )
            })
            .collect()
    }
}

fn main() -> anyhow::Result<()> {
    println!("Ładowanie danych o demografii.");
    let ages = age_distribution();
    let total_population = ages.population();

    println!("Ładowanie danych o szczepieniach.");
    let vaccinations = VaccinationData::new();

    println!("Ładowanie danych o zgonach.");
    let deaths = DeathsData::new();

    println!("Ładowanie danych o infekcjach.");
    let cases = InfectionsData::new();

    println!("Populacja ogólna: {}", total_population);
    println!("Zgonów COVID-19: {}", deaths.total_deaths);

    let weekly_report = |week: YearWeek, age_group: AgeGroup| -> WeeklyReport {
        let population = ages.population_of(age_group);
        let vaccinated_people = vaccinations.sum(age_group.0, week);
        let unvaccinated_people = population - vaccinated_people.at_least_one_dose;

        let absolute_deaths = DeathRate {
            unvaccinated: deaths.by_vaccination_status(
                week,
                age_group,
                VaccinationStatus::Unvaccinated,
            ),
            two_doses: deaths.by_vaccination_status(week, age_group, VaccinationStatus::TwoDoses),
            three_doses: deaths.by_vaccination_status(
                week,
                age_group,
                VaccinationStatus::ThreeDoses,
            ),
        };

        let absolute_cases = DeathRate {
            unvaccinated: cases.by_vaccination_status(
                week,
                age_group,
                VaccinationStatus::Unvaccinated,
            ),
            two_doses: cases.by_vaccination_status(week, age_group, VaccinationStatus::TwoDoses),
            three_doses: cases.by_vaccination_status(
                week,
                age_group,
                VaccinationStatus::ThreeDoses,
            ),
        };

        let deaths_per_million = DeathRate {
            unvaccinated: per_million(absolute_deaths.unvaccinated, unvaccinated_people),
            two_doses: per_million(absolute_deaths.two_doses, vaccinated_people.two_doses),
            three_doses: per_million(absolute_deaths.three_doses, vaccinated_people.three_doses),
        };

        let cases_per_million = DeathRate {
            unvaccinated: per_million(absolute_cases.unvaccinated, unvaccinated_people),
            two_doses: per_million(absolute_cases.two_doses, vaccinated_people.two_doses),
            three_doses: per_million(absolute_cases.three_doses, vaccinated_people.three_doses),
        };

        WeeklyReport {
            vaccinated_people,
            unvaccinated_people,
            absolute_cases,
            absolute_deaths,
            cases_per_million,
            deaths_per_million,
        }
    };

    for age_group in AGE_GROUPS {
        let weekly_reports =
            weeks(cases.max_week()).map(|week| (week, weekly_report(week, *age_group)));
        println!(
            "Grupa wiekowa {} (populacja: {})",
            age_group,
            ages.population_of(*age_group)
        );
        tables::print_stats_for_age_group(*age_group, weekly_reports);
        println!("");
    }

    let weekly_reports_per_age_group = WeeklyReports(
        weeks_of_2021()
            .map(|week| {
                (
                    week,
                    HashMap::<AgeGroup, WeeklyReport>::from_iter(
                        AGE_GROUPS
                            .iter()
                            .map(|age_group| (*age_group, weekly_report(week, *age_group))),
                    ),
                )
            })
            .collect(),
    );

    plots::draw_deaths(&weekly_reports_per_age_group);
    plots::draw_deaths_per_million_per_vaccination_status(&weekly_reports_per_age_group);
    plots::draw_weekly_vaccinations(&vaccinations)?;
    plots::draw_risk_ratios(&weekly_reports_per_age_group);

    plots::draw_vaccinations_one_dose(&weekly_reports_per_age_group);
    plots::draw_vaccinations_two_doses(&weekly_reports_per_age_group);
    plots::draw_vaccinations_at_least_two_doses(&weekly_reports_per_age_group);

    Ok(())
}
