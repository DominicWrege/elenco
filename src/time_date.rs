use chrono::{DateTime, Duration, Utc};
use serde::Serializer;

// after RFC https://tools.ietf.org/html/rfc822

pub fn parse_datetime_rfc822(stamp: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc2822(stamp).map(|t| t.into())
}

fn digit_thing(s: &str) -> Option<i64> {
    match s.len() {
        2 | 3 => match s {
            "00" => Some(0),
            _ => s.trim_start_matches('0').parse::<u16>().ok().map(i64::from),
        },
        1 => s.parse().ok(),
        _ => None,
    }
}

pub trait DurationFormator {
    fn format_duration(&self) -> String {
        if let Some(seconds) = self.duration() {
            let duration = chrono::Duration::seconds(seconds as i64);
            let seconds = duration.num_seconds() % 60;
            let minutes = (duration.num_seconds() / 60) % 60;
            let hours = duration.num_hours();

            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            "None".to_string()
        }
    }
    fn duration(&self) -> Option<i64>;
}

pub fn parse_duration_from_str(s: &str) -> Option<Duration> {
    match s.parse::<i64>() {
        Ok(sec) => Some(Duration::seconds(sec)),
        Err(_) => {
            let digits = s.split(':').collect::<Vec<_>>();
            let (h, m, s) = match digits.as_slice() {
                [h, m, s] if s.len() == 2 => (Some(h), m, s),
                [m, s] if s.len() == 2 => (None, m, s),
                _ => return None,
            };
            let mut hours = digit_thing(h.unwrap_or(&"0"))?;
            let mut minutes = digit_thing(m)?;
            let mut seconds = digit_thing(s)?;
            if seconds >= 60 {
                minutes = minutes + (seconds / 60);
                seconds %= 60;
            };
            if 60 <= minutes {
                hours = minutes / 60;
                minutes %= 60;
            };
            Some(Duration::hours(hours) + Duration::minutes(minutes) + Duration::seconds(seconds))
        }
    }
}

pub fn serialize_datetime<S>(date: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&date.to_rfc3339())
}

pub fn serialize_option_datetime<S>(date: &Option<DateTime<Utc>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(date) => s.serialize_str(&date.to_rfc3339()),
        _ => s.serialize_none(),
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_utc_datetime() {
        let datetimes = [
            "Fri, 23 Oct 2020 03:00:00 -0000",
            "Tue, 17 Jul 2018 03:00:00 +0000",
            "Mon, 23 Nov 2020 22:07:46 GMT",
            "Wed, 18 Nov 2020 11:00:00 -0000",
            "Wed, 25 Nov 2020 17:00:00 +0000",
            "Tue, 24 Nov 2020 05:00:00 +0000",
            "Sun, 08 Nov 2020 13:48:44 GMT",
            "Wed, 20 Mar 2019 16:12:26 +0000",
        ];

        for time in &datetimes {
            parse_datetime_rfc822(time).expect(time);
        }
    }
    //The duration should be in one of the following formats: HH:MM:SS, H:MM:SS, MM:SS, M:SS and MMM::SS
    #[test]
    fn test_duration() {
        let ok = [
            ("00:00", Duration::seconds(0)),
            ("1:00:00", Duration::hours(1)),
            ("01:00:00", Duration::hours(1)),
            (
                "143:45",
                Duration::hours(2) + Duration::minutes(23) + Duration::seconds(45),
            ),
            (
                "218:11",
                Duration::hours(3) + Duration::minutes(38) + Duration::seconds(11),
            ),
            ("60:00", Duration::hours(1)),
            ("02:30:00", Duration::hours(2) + Duration::minutes(30)),
            ("360:00", Duration::hours(6)),
            (
                "12:45:05",
                Duration::hours(12) + Duration::minutes(45) + Duration::seconds(5),
            ),
            ("00:03", Duration::seconds(3)),
            ("27:19", Duration::minutes(27) + Duration::seconds(19)),
            ("0:03", Duration::seconds(3)),
            ("00:44:38", Duration::minutes(44) + Duration::seconds(38)),
            ("0:44:38", Duration::minutes(44) + Duration::seconds(38)),
        ];
        let fail = ["90", "90:999", "00:420", "000:420", "90:210", "7:1", "0:0"];

        for (time, exp) in &ok {
            assert_eq!(
                parse_duration_from_str(time),
                Some(*exp),
                "parsed from: {}",
                time
            );
        }

        for time in &fail {
            assert_eq!(parse_duration_from_str(time), None, "parsed from: {}", time);
        }
    }
}
