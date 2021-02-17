use chrono::{Date, Datelike, Local, TimeZone};

#[derive(Clone, Debug, PartialEq)]
pub struct XLSEvent {
    subject_code: String,
    group: String,
    day: String,
    time: String,
    campus: String,
    location: String,
    duration: String,
    dates: String,
}

impl XLSEvent {
    pub fn new(
        subject_code: String,
        group: String,
        day: String,
        time: String,
        campus: String,
        location: String,
        duration: String,
        dates: String,
    ) -> Self {
        return Self {
            subject_code,
            group,
            day,
            time,
            campus,
            location,
            duration,
            dates,
        };
    }

    pub fn location(&self) -> &String {
        return &self.location;
    }

    pub fn campus(&self) -> &String {
        return &self.campus;
    }

    pub fn duration_in_mins(&self) -> Option<usize> {
        let mut numerics = Vec::new();
        let chars: Vec<char> = self.duration.chars().collect();
        let mut i = 0;

        loop {
            if i >= chars.len() {
                break;
            }

            if chars[i].is_numeric() || chars[i] == '.' {
                numerics.push(chars[i]);
            } else if chars[i].is_whitespace() {
                i += 1;
                break;
            }

            i += 1;
        }

        let duration_identifier = chars[i..].iter().collect::<String>().to_lowercase();
        let mut duration_numeric: f64 = numerics.iter().collect::<String>().parse::<f64>().ok()?;

        if duration_identifier == "hrs" || duration_identifier == "hr" {
            duration_numeric *= 60f64;
        }

        return Some(duration_numeric as usize);
    }

    pub fn get_duration(&self) -> &String {
        return &self.duration;
    }

    pub fn create_name(&self) -> String {
        let subject_code = if self.subject_code.len() > 7 {
            self.subject_code[0..7].to_string()
        } else {
            self.subject_code.clone()
        };

        return format!("{} {}", subject_code, &self.group);
    }

    pub fn get_dates(&self) -> Option<Vec<(Date<Local>, Option<Date<Local>>)>> {
        return Self::dates_from_string(self.dates.clone());
    }

    pub fn get_time(&self) -> Option<(u32, u32)> {
        return Self::time_from_string(self.time.clone());
    }

    fn dates_from_string(str: String) -> Option<Vec<(Date<Local>, Option<Date<Local>>)>> {
        let mut components = Vec::new();
        let mut current_component = Vec::new();

        for char in str.chars() {
            if char.is_whitespace() {
                continue;
            } else if char == ',' {
                components.push(current_component);
                current_component = Vec::new();
            } else if char == '/' || char == '-' || char.is_numeric() {
                current_component.push(char);
            } else {
                return None;
            }
        }

        if !current_component.is_empty() {
            components.push(current_component);
        }

        let mut res = Vec::new();

        fn date_from_pair(chars: Vec<char>) -> Option<Date<Local>> {
            let mut slash_position = None;

            for (i, ch) in chars.iter().enumerate() {
                if *ch == '/' {
                    slash_position = Some(i);
                    break;
                }
            }

            let slash_position = slash_position?;

            if slash_position > 2 {
                return None;
            }

            let day = chars[0..slash_position]
                .iter()
                .collect::<String>()
                .parse::<u32>()
                .ok()?;

            let month = chars[slash_position + 1..].iter().collect::<String>();

            if month.len() > 2 {
                return None;
            }

            let month = month.parse::<u32>().ok()?;

            return Some(Local.ymd(Local::now().year(), month, day));
        }

        for component in components {
            let mut start_chars = Vec::new();
            let mut end_chars = Vec::new();
            let mut finished_start = false;

            for char in component {
                if char == '-' {
                    finished_start = true;
                } else if finished_start {
                    end_chars.push(char);
                } else {
                    start_chars.push(char);
                }
            }

            if !finished_start {
                res.push((date_from_pair(start_chars)?, None));
            } else {
                res.push((
                    date_from_pair(start_chars)?,
                    Some(date_from_pair(end_chars)?),
                ));
            }
        }

        return Some(res);
    }

    fn time_from_string(str: String) -> Option<(u32, u32)> {
        if str.len() != 5 || str.chars().nth(2) != Some(':') {
            return None;
        }

        let hour = str[..2].parse::<u32>().ok()?;
        let minute = str[3..].parse::<u32>().ok()?;

        return Some((hour, minute));
    }
}

#[cfg(test)]
mod tests {
    use super::XLSEvent;
    use chrono::{Datelike, Local, TimeZone};

    #[test]
    fn test_dates_from_string() {
        let date = "5/3-2/4, 16/4-28/5".to_string();
        assert_eq!(
            vec![
                (
                    Local.ymd(Local::now().year(), 3, 5),
                    Some(Local.ymd(Local::now().year(), 4, 2))
                ),
                (
                    Local.ymd(Local::now().year(), 4, 16),
                    Some(Local.ymd(Local::now().year(), 5, 28))
                )
            ],
            XLSEvent::dates_from_string(date).unwrap()
        );
    }

    #[test]
    fn test_time_from_string() {
        let time = "08:32".to_string();
        assert_eq!((8, 32), XLSEvent::time_from_string(time).unwrap());
    }
}
