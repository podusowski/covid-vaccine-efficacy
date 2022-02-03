use std::collections::HashMap;

use calamine::{open_workbook, Reader, Xls};

use crate::AgeGroup;

pub(crate) struct AgeDistribution {
    ages: HashMap<usize, usize>,
}

impl AgeDistribution {
    pub(crate) fn population(&self) -> usize {
        self.ages.values().sum()
    }

    pub(crate) fn population_of(&self, age_group: AgeGroup) -> usize {
        self.ages
            .iter()
            .map(|(age, count)| if age_group.includes(*age) { *count } else { 0 })
            .sum()
    }
}

pub(crate) fn age_distribution() -> AgeDistribution {
    let mut workbook: Xls<_> = open_workbook("data/tabela01.xls").unwrap();
    let range = workbook.worksheet_range("Tabl. 1").unwrap().unwrap();
    let mut ages = HashMap::<usize, usize>::new();

    for row in range.rows() {
        if let Some(age) = row[0].get_float() {
            let value: usize = match row[1] {
                calamine::DataType::Int(x) => x as usize,
                calamine::DataType::Float(x) => x as usize,
                _ => panic!("can't interpret that"),
            };
            assert!(ages.insert(age as usize, value) == None);
        }
    }

    AgeDistribution { ages }
}
