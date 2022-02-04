use chrono::{Datelike, NaiveDate};
use plotters::{coord::types::RangedCoordu32, prelude::*};

use crate::{vaccination::VaccinationData, WeeklyReport, WeeklyReports, YearWeek, AGE_GROUPS};

pub(crate) fn draw_weekly_vaccinations(people_vaccinated: &VaccinationData) -> anyhow::Result<()> {
    let path = format!("output/vaccinated_people.png");
    let area = BitMapBackend::new(path.as_str(), (1024, 400)).into_drawing_area();

    let weeks: Vec<YearWeek> = NaiveDate::from_ymd(2021, 1, 1)
        .iter_weeks()
        .take(52)
        .map(|date| date.iso_week().into())
        .collect();

    let x_axis = 0usize..(weeks.len() - 1);
    let y_axis = 0usize..10_000_000usize;

    let caption = format!("Ilość zaszczepionych osób (50-59) przynajmniej 2 dawkami");

    area.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&area)
        .caption(
            caption.clone(),
            ("sans-serif", 12).into_font().color(&BLACK),
        )
        .set_label_area_size(LabelAreaPosition::Left, 12.percent())
        .set_label_area_size(LabelAreaPosition::Bottom, 10.percent())
        .margin(1.percent())
        .build_cartesian_2d(x_axis.clone(), y_axis.clone())?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("Tydzień")
        .y_desc("Ilość zaszczepionych osób")
        .x_label_formatter(&|n| format!("{:?}", weeks[*n as usize]))
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            weeks
                .iter()
                .enumerate()
                .map(|(n, week)| (n, people_vaccinated.sum((50, 59), *week).at_least_two_doses)),
            RED.stroke_width(2),
        ))?
        .label("dwie dawki")
        .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], RED.filled()));

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperMiddle)
        .border_style(&BLACK)
        .draw()?;

    area.present()?;

    Ok(())
}

/// Draws generic chart based on a weekly data.
fn quick_weekly_chart(
    reports: &WeeklyReports,
    path: String,
    caption: String,
    y_desc: String,
    max_x: u32,
    draw: impl Fn(&mut ChartContext<SVGBackend, Cartesian2d<RangedCoordu32, RangedCoordu32>>),
) {
    let area = SVGBackend::new(path.as_str(), (1024, 400)).into_drawing_area();
    area.fill(&WHITE).unwrap();

    let x_axis = 0u32..(reports.0.len() - 1) as u32;
    let y_axis = 0u32..max_x;

    let mut chart = ChartBuilder::on(&area)
        .caption(
            caption.clone(),
            ("sans-serif", 16).into_font().color(&BLACK),
        )
        .set_label_area_size(LabelAreaPosition::Left, 12.percent())
        .set_label_area_size(LabelAreaPosition::Bottom, 10.percent())
        .margin(1.percent())
        .build_cartesian_2d(x_axis.clone(), y_axis.clone())
        .unwrap();

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("Tydzień")
        .y_desc(y_desc)
        .x_label_formatter(&|n| format!("{}", reports.0[*n as usize].0))
        .draw()
        .unwrap();

    draw(&mut chart);

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperMiddle)
        .border_style(&BLACK)
        .draw()
        .unwrap();

    area.present().unwrap();
}

pub(crate) fn draw_deaths(reports: &WeeklyReports) {
    quick_weekly_chart(
        reports,
        "output/deaths.svg".to_owned(),
        "Zgony w poszczególnych grupach wiekowych (liczby bezwzględne)".to_owned(),
        "Zgony".to_owned(),
        1200,
        |chart| {
            for (chart_idx, age_group) in AGE_GROUPS.iter().enumerate() {
                let color = Palette99::pick(chart_idx);
                chart
                    .draw_series(LineSeries::new(
                        reports.0.iter().enumerate().map(|(n, (_, report))| {
                            (
                                n as u32,
                                report.get(&age_group).unwrap().absolute_deaths.total() as u32,
                            )
                        }),
                        color.stroke_width(2),
                    ))
                    .unwrap()
                    .label(format!("{age_group}"))
                    .legend(move |(x, y)| {
                        Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                    });
            }
        },
    );
}

pub(crate) fn draw_risk_ratios(reports: &WeeklyReports) {
    quick_weekly_chart(
        reports,
        "output/risk_ratios.svg".to_owned(),
        "Ryzyko względne zgonu osób zaszczepionych (%)".to_owned(),
        "%".to_owned(),
        100,
        |chart| {
            let color = Palette99::pick(0);
            chart
                .draw_series(LineSeries::new(
                    reports
                        .mean(WeeklyReport::risk_ratio_of_two_doses)
                        .iter()
                        .enumerate()
                        .map(|(n, (_, rr))| (n as u32, (rr * 100f64) as u32)),
                    color.stroke_width(2),
                ))
                .unwrap()
                .label("2 dawki")
                .legend(move |(x, y)| {
                    Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                });

            let color = Palette99::pick(1);
            chart
                .draw_series(LineSeries::new(
                    reports
                        .mean(WeeklyReport::risk_ratio_of_three_doses)
                        .iter()
                        .enumerate()
                        .map(|(n, (_, rr))| (n as u32, (rr * 100f64) as u32)),
                    color.stroke_width(2),
                ))
                .unwrap()
                .label("3 dawki")
                .legend(move |(x, y)| {
                    Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                });
        },
    );
}

pub(crate) fn draw_case_risk_ratios(reports: &WeeklyReports) {
    quick_weekly_chart(
        reports,
        "output/infection_risk_ratios.svg".to_owned(),
        "Ryzyko względne pozytywnego testu u osób zaszczepionych (%)".to_owned(),
        "%".to_owned(),
        200,
        |chart| {
            let color = Palette99::pick(0);
            chart
                .draw_series(LineSeries::new(
                    reports
                        .mean(WeeklyReport::case_risk_ratio_of_two_doses)
                        .iter()
                        .enumerate()
                        .map(|(n, (_, rr))| (n as u32, (rr * 100f64) as u32)),
                    color.stroke_width(2),
                ))
                .unwrap()
                .label("2 dawki")
                .legend(move |(x, y)| {
                    Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                });

            let color = Palette99::pick(1);
            chart
                .draw_series(LineSeries::new(
                    reports
                        .mean(WeeklyReport::case_risk_ratio_of_three_doses)
                        .iter()
                        .enumerate()
                        .map(|(n, (_, rr))| (n as u32, (rr * 100f64) as u32)),
                    color.stroke_width(2),
                ))
                .unwrap()
                .label("3 dawki")
                .legend(move |(x, y)| {
                    Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                });
        },
    );
}

pub(crate) fn draw_deaths_per_million_per_vaccination_status(reports: &WeeklyReports) {
    quick_weekly_chart(
        reports,
        "output/deaths_per_vaccination_status.svg".to_owned(),
        "Zgony w przeliczeniu na million mieszkańców".to_owned(),
        "Zgony".to_owned(),
        1500,
        |chart| {
            let color = Palette99::pick(0);
            chart
                .draw_series(LineSeries::new(
                    reports.0.iter().enumerate().map(|(n, (_, report))| {
                        (
                            n as u32,
                            report
                                .values()
                                .map(|weekly_report| weekly_report.deaths_per_million.unvaccinated)
                                .sum::<f64>() as u32,
                        )
                    }),
                    color.stroke_width(2),
                ))
                .unwrap()
                .label(format!("niezaszczepieni"))
                .legend(move |(x, y)| {
                    Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                });

            let color = Palette99::pick(1);
            chart
                .draw_series(LineSeries::new(
                    reports.0.iter().enumerate().map(|(n, (_, report))| {
                        (
                            n as u32,
                            report
                                .values()
                                .map(|weekly_report| weekly_report.deaths_per_million.two_doses)
                                .sum::<f64>() as u32,
                        )
                    }),
                    color.stroke_width(2),
                ))
                .unwrap()
                .label(format!("2 dawki"))
                .legend(move |(x, y)| {
                    Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                });

            let color = Palette99::pick(2);
            chart
                .draw_series(LineSeries::new(
                    reports.0.iter().enumerate().map(|(n, (_, report))| {
                        (
                            n as u32,
                            report
                                .values()
                                .map(|weekly_report| weekly_report.deaths_per_million.three_doses)
                                .sum::<f64>() as u32,
                        )
                    }),
                    color.stroke_width(2),
                ))
                .unwrap()
                .label(format!("3 dawki"))
                .legend(move |(x, y)| {
                    Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                });
        },
    );
}

pub(crate) fn draw_vaccinations_one_dose(reports: &WeeklyReports) {
    quick_weekly_chart(
        reports,
        "output/vaccinations_one_dose.svg".to_owned(),
        "Ilość osób zaszczepionych 1 dawką".to_owned(),
        "Ilość".to_owned(),
        10_000_000,
        |chart| {
            for (chart_idx, age_group) in AGE_GROUPS.iter().enumerate() {
                let color = Palette99::pick(chart_idx);
                chart
                    .draw_series(LineSeries::new(
                        reports.0.iter().enumerate().map(|(n, (_, report))| {
                            (
                                n as u32,
                                report.get(&age_group).unwrap().vaccinated_people.one_dose as u32,
                            )
                        }),
                        color.stroke_width(2),
                    ))
                    .unwrap()
                    .label(format!("{age_group}"))
                    .legend(move |(x, y)| {
                        Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                    });
            }
        },
    );
}

pub(crate) fn draw_vaccinations_two_doses(reports: &WeeklyReports) {
    quick_weekly_chart(
        reports,
        "output/vaccinations_two_doses.svg".to_owned(),
        "Ilość osób zaszczepionych 2 dawkami".to_owned(),
        "Ilość".to_owned(),
        10_000_000,
        |chart| {
            for (chart_idx, age_group) in AGE_GROUPS.iter().enumerate() {
                let color = Palette99::pick(chart_idx);
                chart
                    .draw_series(LineSeries::new(
                        reports.0.iter().enumerate().map(|(n, (_, report))| {
                            (
                                n as u32,
                                report.get(&age_group).unwrap().vaccinated_people.two_doses as u32,
                            )
                        }),
                        color.stroke_width(2),
                    ))
                    .unwrap()
                    .label(format!("{age_group}"))
                    .legend(move |(x, y)| {
                        Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                    });
            }
        },
    );
}

pub(crate) fn draw_vaccinations_at_least_two_doses(reports: &WeeklyReports) {
    quick_weekly_chart(
        reports,
        "output/vaccinations_at_least_two_doses.svg".to_owned(),
        "Ilość osób zaszczepionych co najmniej 2 dawkami".to_owned(),
        "Ilość".to_owned(),
        10_000_000,
        |chart| {
            for (chart_idx, age_group) in AGE_GROUPS.iter().enumerate() {
                let color = Palette99::pick(chart_idx);
                chart
                    .draw_series(LineSeries::new(
                        reports.0.iter().enumerate().map(|(n, (_, report))| {
                            (
                                n as u32,
                                report
                                    .get(&age_group)
                                    .unwrap()
                                    .vaccinated_people
                                    .at_least_two_doses as u32,
                            )
                        }),
                        color.stroke_width(2),
                    ))
                    .unwrap()
                    .label(format!("{age_group}"))
                    .legend(move |(x, y)| {
                        Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                    });
            }
        },
    );
}
