//! Model for serialized ISO weeks in persistent storage

use crate::schedule::Error;
use chrono::offset::LocalResult;
use chrono::{DateTime, TimeZone, Utc, Weekday};
use diesel::deserialize::{FromSql, Result as FromSqlResult};
use diesel::serialize::{Output, Result as ToSqlResult, ToSql};
use diesel::{backend::Backend, sql_types::Integer};
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};
use std::io::Write;

/// Model for serialized ISO weeks in persistent storage
#[derive(AsExpression, Debug, Deserialize, FromSqlRow, PartialEq)]
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
            .ok_or_else(|| Error::InvalidStartWeekYear(self.year))?;

        let serialized_iso_week = shifted_year
            .checked_add(self.week)
            .ok_or_else(|| Error::InvalidStartWeekWeek(self.week))?;

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

impl TryInto<DateTime<Utc>> for StoredIsoWeek {
    type Error = Error;

    /// Converts from a `StoredIsoWeek` to chrono's `DateTime<Utc>` type
    fn try_into(self) -> Result<DateTime<Utc>, Self::Error> {
        let week = u32::try_from(self.week).map_err(|_| Error::InvalidStartWeekWeek(self.week))?;
        let year = self.year;

        match Utc
            .isoywd_opt(year, week, Weekday::Mon)
            .and_hms_opt(0, 0, 0)
        {
            LocalResult::None => Err(Error::InvalidStartWeek { week, year }),
            // 0h0m0s on Monday of a valid week can't be ambiguous, but just in case...
            LocalResult::Ambiguous(first, second) => Err(Error::AmbiguousStartWeek {
                week,
                year,
                first,
                second,
            }),
            LocalResult::Single(start_date) => Ok(start_date),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_to_utc_datetime() {
        let iso_week = StoredIsoWeek {
            year: 2020,
            week: 2,
        };

        let expected_datetime = Utc.ymd(2020, 1, 6).and_hms(0, 0, 0);

        assert_eq!(Ok(expected_datetime), iso_week.try_into());
    }

    #[test]
    fn it_fails_to_convert_to_invalid_utc_datetime() {
        let iso_week = StoredIsoWeek {
            year: 2020,
            week: 256,
        };

        let expected_error: Result<DateTime<Utc>, _> = Err(Error::InvalidStartWeek {
            week: 256,
            year: 2020,
        });

        assert_eq!(expected_error, iso_week.try_into());
    }
}
