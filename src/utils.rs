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

use chrono::NaiveDate;
use objects::Date;

pub fn de_from_u8<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: ::serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let i = u8::deserialize(deserializer)?;
    Ok(i != 0)
}

pub fn ser_from_bool<S>(v: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ::serde::Serializer,
{
    serializer.serialize_u8(*v as u8)
}

pub fn de_from_date_string<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: ::serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s = String::deserialize(deserializer)?;

    NaiveDate::parse_from_str(&s, "%Y%m%d").map_err(::serde::de::Error::custom)
}

pub fn ser_from_naive_date<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ::serde::Serializer,
{
    let s = format!("{}", date.format("%Y%m%d"));
    serializer.serialize_str(&s)
}

pub fn de_with_empty_default<'de, T: Default, D>(de: D) -> Result<T, D::Error>
where
    D: ::serde::Deserializer<'de>,
    for<'d> T: ::serde::Deserialize<'d>,
{
    use serde::Deserialize;
    Option::<T>::deserialize(de).map(|opt| opt.unwrap_or_else(Default::default))
}
