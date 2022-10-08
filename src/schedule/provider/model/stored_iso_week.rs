//! Model for serialized ISO weeks in persistent storage

use crate::schedule::Error;
use diesel::deserialize::{FromSql, Result as FromSqlResult};
use diesel::serialize::{Output, Result as ToSqlResult, ToSql};
use diesel::{backend::Backend, sql_types::Integer};
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};
use std::io::Write;
use time::{util::weeks_in_year, Date, OffsetDateTime, Weekday};

/// Model for serialized ISO weeks in persistent storage
#[derive(AsExpression, Debug, Deserialize, Eq, FromSqlRow, PartialEq)]
#[sql_type = "Integer"]
pub struct StoredIsoWeek {
    /// Year of the ISO week
    pub year: i32,
    /// Week of the year, between 1 and 53 inclusive
    pub week: i32,
}

impl<TDatabase: Backend> ToSql<Integer, TDatabase> for StoredIsoWeek
where
    i32: ToSql<Integer, TDatabase>,
{
    /// Converts this model to a SQL type
    /// Serialization is performed by converting year 2020 week 33 to 202033
    fn to_sql<W: Write>(&self, out: &mut Output<W, TDatabase>) -> ToSqlResult {
        let shifted_year = self
            .year
            .checked_mul(100)
            .ok_or(Error::YearTooLarge(self.year))?;

        let serialized_iso_week = shifted_year
            .checked_add(self.week)
            .ok_or(Error::WeekTooLarge(self.week))?;

        serialized_iso_week.to_sql(out)
    }
}

impl<TDatabase: Backend> FromSql<Integer, TDatabase> for StoredIsoWeek
where
    i32: FromSql<Integer, TDatabase>,
{
    /// Creates this model from a SQL type
    /// Iso weeks are serialized as 202033, where 2020 is the year and 33 is the week
    fn from_sql(bytes: Option<&TDatabase::RawValue>) -> FromSqlResult<Self> {
        let serialized_iso_week = i32::from_sql(bytes)?;
        Ok(Self {
            year: serialized_iso_week / 100,
            week: serialized_iso_week % 100,
        })
    }
}

impl TryInto<OffsetDateTime> for StoredIsoWeek {
    type Error = Error;

    /// Converts from a `StoredIsoWeek` to time's `OffsetDateTime` type
    fn try_into(self) -> Result<OffsetDateTime, Self::Error> {
        let week = u8::try_from(self.week)
            .ok()
            .filter(|week| week <= &weeks_in_year(self.year))
            .ok_or(Error::WeekTooLarge(self.week))?;
        let year = self.year;

        match Date::from_iso_week_date(year, week, Weekday::Monday) {
            Ok(start_date) => Ok(start_date.midnight().assume_utc()),
            _ => Err(Error::InvalidStartWeek {
                week: u32::from(week),
                year,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn it_converts_to_utc_datetime() {
        let iso_week = StoredIsoWeek {
            year: 2020,
            week: 2,
        };

        let expected_datetime = datetime!(2020-01-06 00:00:00 UTC);

        assert_eq!(Ok(expected_datetime), iso_week.try_into());
    }

    #[test]
    fn it_fails_to_convert_to_invalid_utc_datetime() {
        let iso_week = StoredIsoWeek {
            year: 2020,
            week: 256,
        };

        let expected_error: Result<OffsetDateTime, _> = Err(Error::WeekTooLarge(256));

        assert_eq!(expected_error, iso_week.try_into());
    }
}
