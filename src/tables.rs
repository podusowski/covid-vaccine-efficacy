use prettytable::{Cell, Row};

use crate::{AgeGroup, WeeklyReport, YearWeek};

pub(crate) fn print_stats_for_age_group(
    age_group: AgeGroup,
    weekly_reports: impl Iterator<Item = (YearWeek, WeeklyReport)>,
) {
    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_CLEAN);

    let data: Vec<(&str, &dyn Fn(YearWeek, WeeklyReport) -> String)> = vec![
        ("Tydzień", &|week, _| format!("{}", week)),
        ("Niezasz.", &|_, report| {
            format!("{}", report.unvaccinated_people)
        }),
        ("1", &|_, report| {
            format!("{}", report.vaccinated_people.one_dose)
        }),
        ("2", &|_, report| {
            format!("{}", report.vaccinated_people.two_doses)
        }),
        ("3", &|_, report| {
            format!("{}", report.vaccinated_people.three_doses)
        }),
        ("1+", &|_, report| {
            format!("{}", report.vaccinated_people.at_least_one_dose)
        }),
        ("2+", &|_, report| {
            format!("{}", report.vaccinated_people.at_least_two_doses)
        }),
        ("Zak. (NZ)", &|_, report| {
            format!("{}", report.absolute_cases.unvaccinated)
        }),
        ("Zak. (2)", &|_, report| {
            format!("{}", report.absolute_cases.two_doses)
        }),
        ("Zak. (3)", &|_, report| {
            format!("{}", report.absolute_cases.three_doses)
        }),
        // Cases per million
        ("Zak./mln (NZ)", &|_, report| {
            format!("{:.2}", report.cases_per_million.unvaccinated)
        }),
        ("Zak./mln (2)", &|_, report| {
            format!("{:.2}", report.cases_per_million.two_doses)
        }),
        ("Zak./mln (3)", &|_, report| {
            format!("{:.2}", report.cases_per_million.three_doses)
        }),
        // Deaths
        ("Zg. (NZ)", &|_, report| {
            format!("{}", report.absolute_deaths.unvaccinated)
        }),
        ("Zg. (2)", &|_, report| {
            format!("{}", report.absolute_deaths.two_doses)
        }),
        ("Zg. (3)", &|_, report| {
            format!("{}", report.absolute_deaths.three_doses)
        }),
        // Deaths per mln
        ("Zg./mln (NZ)", &|_, report| {
            format!("{:.2}", report.deaths_per_million.unvaccinated)
        }),
        ("Zg./mln (2)", &|_, report| {
            format!("{:.2}", report.deaths_per_million.two_doses)
        }),
        ("Zg./mln (3)", &|_, report| {
            format!("{:.2}", report.deaths_per_million.three_doses)
        }),
        // RR of case
        ("RR zak. (2)", &|_, report| {
            format!("{:.2}", report.case_risk_ratio_of_two_doses())
        }),
        ("RR zak. (3)", &|_, report| {
            format!("{:.2}", report.case_risk_ratio_of_three_doses())
        }),
        // RR of death
        ("RR zg. (2)", &|_, report| {
            format!("{:.2}", report.risk_ratio_of_two_doses())
        }),
        ("RR zg. (3)", &|_, report| {
            format!("{:.2}", report.risk_ratio_of_three_doses())
        }),
        // CFR
        ("CFR (NZ)", &|_, report| {
            format!("{:.3}", report.cfr_unvaccinated())
        }),
        ("CFR (2)", &|_, report| {
            format!("{:.3}", report.cfr_two_doses())
        }),
    ];

    table.add_row(Row::new(data.iter().map(|row| Cell::new(row.0)).collect()));

    for (week, report) in weekly_reports {
        table.add_row(Row::new(
            data.iter()
                .map(|row| Cell::new(row.1(week, report).as_str()))
                .collect(),
        ));
    }
    table.print_tty(false);

    let csv = std::fs::File::create(format!(
        "output/details_for_{}_{}.csv",
        age_group.0 .0, age_group.0 .1
    ))
    .unwrap();
    table.to_csv(csv).unwrap();
}
