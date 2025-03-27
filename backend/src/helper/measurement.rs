use crate::api::{
    common::{Series, SeriesGranularity, SeriesItem},
    measurement::Measurement,
};

use chrono::{Duration, NaiveTime};

/// Given a list of measurements, it aggregates them at the specified granularity
/// measurement - The measurements to aggregate. Sorted by recorded_at, descending
///               (starting with the most recent measurements)
/// granularity - The granularity at which the measurements will be aggregated
pub fn create_series(measurements: &Vec<Measurement>, granularity: SeriesGranularity) -> Series {
    let mut items = vec![];

    if measurements.len() == 0 {
        return Series { granularity, items };
    }

    let mut i = 0;
    let mut current_start =
        measurements[i].recorded_at.date().and_time(NaiveTime::MIN) + Duration::days(1);
    while i < measurements.len() {
        let mut total = 0.0;
        while i < measurements.len() && measurements[i].recorded_at > current_start {
            total = total + measurements[i].measurement.parse::<f64>().unwrap();
            i = i + 1;
        }

        if total != 0.0 {
            items.push(SeriesItem {
                period_start: current_start,
                value: total.to_string(),
            });
        }

        match granularity {
            SeriesGranularity::Month => {
                // TODO: This is not really accurate, but we also don't want to
                // do month aggregation at run-time
                current_start = current_start - Duration::days(30);
            }
            SeriesGranularity::Day => {
                current_start = current_start - Duration::days(1);
            }
            SeriesGranularity::Hour => {
                current_start = current_start - Duration::hours(1);
            }
        }
    }

    return Series { granularity, items };
}

#[cfg(test)]
mod tests {
    use super::create_series;
    use crate::api::{
        common::{Series, SeriesGranularity, SeriesItem},
        measurement::Measurement,
    };

    use chrono::{Duration, NaiveDateTime};
    use uuid::Uuid;

    #[test]
    fn create_series_success() {
        let datetime =
            NaiveDateTime::parse_from_str("2025-03-27 14:42:32", "%Y-%m-%d %H:%M:%S").unwrap();
        let measuments = vec![
            Measurement {
                id: Uuid::new_v4().to_string(),
                device_id: "device_id".to_string(),
                measurement: 5.5.to_string(),
                recorded_at: datetime,
            },
            Measurement {
                id: Uuid::new_v4().to_string(),
                device_id: "device_id".to_string(),
                measurement: 1.to_string(),
                recorded_at: datetime - Duration::minutes(40),
            },
            Measurement {
                id: Uuid::new_v4().to_string(),
                device_id: "device_id".to_string(),
                measurement: 22.3.to_string(),
                recorded_at: datetime - Duration::minutes(60),
            },
            Measurement {
                id: Uuid::new_v4().to_string(),
                device_id: "device_id".to_string(),
                measurement: 1.to_string(),
                recorded_at: datetime - Duration::minutes(80),
            },
            Measurement {
                id: Uuid::new_v4().to_string(),
                device_id: "device_id".to_string(),
                measurement: 2.to_string(),
                recorded_at: datetime - Duration::minutes(100),
            },
            Measurement {
                id: Uuid::new_v4().to_string(),
                device_id: "device_id".to_string(),
                measurement: 3.to_string(),
                recorded_at: datetime - Duration::minutes(120),
            },
        ];
        let res = create_series(&measuments, SeriesGranularity::Hour);

        let mut items = vec![];
        let mut hour =
            NaiveDateTime::parse_from_str("2025-03-27 14:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        items.push(SeriesItem {
            period_start: hour,
            value: "6.5".to_string(),
        });
        hour = hour - Duration::hours(1);
        items.push(SeriesItem {
            period_start: hour,
            value: "25.3".to_string(),
        });
        hour = hour - Duration::hours(1);
        items.push(SeriesItem {
            period_start: hour,
            value: "3".to_string(),
        });
        let expected = Series {
            granularity: SeriesGranularity::Hour,
            items,
        };
        assert_eq!(expected, res);

        let res = create_series(&measuments, SeriesGranularity::Day);

        let mut items = vec![];
        let day =
            NaiveDateTime::parse_from_str("2025-03-27 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        items.push(SeriesItem {
            period_start: day,
            value: "34.8".to_string(),
        });
        let expected = Series {
            granularity: SeriesGranularity::Day,
            items,
        };
        assert_eq!(expected, res);
    }
}
