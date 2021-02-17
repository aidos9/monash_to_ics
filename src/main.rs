mod event;

use calamine::{DataType, Reader, Xls};
use chrono::{DateTime, Duration, Local, Utc};
use clap::{App, Arg};
use event::*;
use ics::properties::{Description, DtEnd, DtStart, Location, RRule, Summary};
use ics::{Event, ICalendar};
use std::io::{stdin, stdout, Write};
use std::process::exit;

const SUBJECT_CODE_COLUMN: usize = 0;
const GROUP_COLUMN: usize = 2;
const DAY_COLUMN: usize = 4;
const TIME_COLUMN: usize = 5;
const CAMPUS_COLUMN: usize = 6;
const LOCATION_COLUMN: usize = 7;
const DURATION_COLUMN: usize = 9;
const DATES_COLUMN: usize = 10;
const LARGEST_COLUMN: usize = 10;

fn main() {
    let matches = App::new("monash_to_ics").help("This application converts monash timetable xls files into ics files for calendar applications.")
        .arg(
            Arg::with_name("file")
                .required(true)
                .takes_value(true)
                .max_values(1)
                .help("The file to process"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .required(true)
                .takes_value(true)
                .default_value("out.ics")
                .help("The output file"),
        )
        .arg(
            Arg::with_name("worksheet")
                .short("w")
                .required(true)
                .takes_value(true)
                .default_value(" ")
                .help("The name of the sheet with the calendar data."),
        )
        .arg(
            Arg::with_name("no_check")
                .long("no-check")
                .help("Don't check the name of the events."),
        )
        .get_matches();

    let mut workbook: Xls<_> =
        calamine::open_workbook(matches.value_of("file").unwrap().to_string())
            .expect("Cannot open file");

    if let Some(Ok(range)) = workbook.worksheet_range(matches.value_of("worksheet").unwrap()) {
        let contents: Vec<Vec<&DataType>> = range
            .rows()
            .map(|row| row.iter().filter(|&cell| !cell.is_empty()).collect())
            .collect();

        println!(
            "Found {} non-empty cells.",
            contents.iter().flatten().count()
        );

        if contents.is_empty() || contents[0].is_empty() {
            return;
        }

        let mut events: Vec<XLSEvent> = contents
            .into_iter()
            .map(|row| {
                if row.len() <= LARGEST_COLUMN {
                    eprintln!(
                        "At least {} columns are required, instead only {} were supplied.",
                        LARGEST_COLUMN,
                        row.len()
                    );

                    exit(1);
                }

                XLSEvent::new(
                    row[SUBJECT_CODE_COLUMN].to_string(),
                    row[GROUP_COLUMN].to_string(),
                    row[DAY_COLUMN].to_string(),
                    row[TIME_COLUMN].to_string(),
                    row[CAMPUS_COLUMN].to_string(),
                    row[LOCATION_COLUMN].to_string(),
                    row[DURATION_COLUMN].to_string(),
                    row[DATES_COLUMN].to_string(),
                )
            })
            .collect();

        // Remove the header row
        events.remove(0);

        let mut calendar = ICalendar::new("2.0", "ics-rs");

        for event in events {
            let event_name;
            let duration_mins;
            let dates;

            println!("\n\nFound event {}", event.create_name());
            print!("Is this name ok(y/n) ");

            if read_bool() {
                event_name = event.create_name();
            } else {
                print!("Event name: ");
                event_name = read_string();
            }

            if let Some(duration) = event.duration_in_mins() {
                println!(
                    "The requested duration was {}, we determined this to be {} minutes",
                    event.get_duration(),
                    duration
                );
                print!("Is this ok?(y/n) ");

                if !read_bool() {
                    print!("Your duration(minutes): ");
                    duration_mins = read_usize();
                } else {
                    duration_mins = duration;
                }
            } else {
                println!("Failed to determine the correct duration in minutes.");
                println!("The reported duration was {}", event.get_duration());
                print!("Your duration(minutes): ");
                duration_mins = read_usize();
            }

            if let Some(chrono_dates) = event.get_dates() {
                let time = if let Some(time) = event.get_time() {
                    time
                } else {
                    eprintln!("Something went wrong whilst processing the times.");
                    exit(1);
                };

                dates = chrono_dates
                    .into_iter()
                    .map(|(d1, d2)| {
                        (
                            d1.and_hms(time.0, time.1, 0),
                            d2.map(|d| d.and_hms(time.0, time.1, 0)),
                        )
                    })
                    .collect::<Vec<(DateTime<Local>, Option<DateTime<Local>>)>>();
            } else {
                eprintln!("Something went wrong whilst processing the dates.");
                exit(1);
            }

            println!(
                "{} takes place in the following ranges, inclusive.",
                &event_name
            );

            for (start, end) in &dates {
                if let Some(end) = end {
                    println!(
                        "{} - {}",
                        start.format("%Y-%m-%d %H:%M:%S"),
                        end.format("%Y-%m-%d %H:%M:%S")
                    );
                } else {
                    println!("{}", start.format("%Y-%m-%d %H:%M:%S"));
                }
            }

            print!("Add event?(y/n) ");

            if !read_bool() {
                continue;
            }

            for (start, end) in dates {
                if let Some(end) = end {
                    let mut ics_event = Event::new(
                        uuid::Uuid::new_v4().to_hyphenated().to_string(),
                        current_dt_stamp(),
                    );

                    ics_event.push(DtStart::new(format_local_into_utc(start)));
                    ics_event.push(DtEnd::new(format_local_into_utc(
                        start + Duration::minutes(duration_mins as i64),
                    )));
                    ics_event.push(Summary::new(event_name.clone()));
                    ics_event.push(Location::new(event.location().clone()));
                    ics_event.push(Description::new(format!("Campus: {}", event.campus())));

                    ics_event.push(RRule::new(format!(
                        "FREQ=WEEKLY;UNTIL={}",
                        format_local_into_utc(end + Duration::minutes(duration_mins as i64 + 10))
                    )));

                    calendar.add_event(ics_event);
                    println!(
                        "Added event {}     {} - {}",
                        event_name,
                        start.format("%Y-%m-%d %H:%M:%S"),
                        end.format("%Y-%m-%d %H:%M:%S")
                    );
                } else {
                    let mut ics_event = Event::new(
                        uuid::Uuid::new_v4().to_hyphenated().to_string(),
                        current_dt_stamp(),
                    );

                    ics_event.push(DtStart::new(format_local_into_utc(start)));
                    ics_event.push(DtEnd::new(format_local_into_utc(
                        start + Duration::minutes(duration_mins as i64),
                    )));
                    ics_event.push(Summary::new(event_name.clone()));
                    ics_event.push(Location::new(event.location().clone()));
                    ics_event.push(Description::new(format!("Campus: {}", event.campus())));

                    calendar.add_event(ics_event);
                    println!(
                        "Added event {}     {}",
                        event_name,
                        start.format("%Y-%m-%d %H:%M:%S")
                    );
                }
            }
        }

        calendar
            .save_file(matches.value_of("output").unwrap())
            .expect("Failed to write iCalendar file.");

        println!(
            "Successfully wrote timetable to {}",
            matches.value_of("output").unwrap()
        );
    } else {
        eprintln!(
            "Failed to find the specified sheet {}",
            matches.value_of("worksheet").unwrap()
        );

        exit(1);
    }
}

fn read_usize() -> usize {
    loop {
        let _ = stdout().flush();
        let mut text = String::new();
        stdin()
            .read_line(&mut text)
            .expect("Something went wrong while reading stdin.");

        text = text
            .strip_suffix("\n")
            .map(|s| s.to_string())
            .unwrap_or(text);

        if let Ok(res) = text.parse::<usize>() {
            return res;
        } else {
            print!("Please enter an integer.\nTry again: ");
        }
    }
}

fn read_bool() -> bool {
    loop {
        let _ = stdout().flush();
        let mut text = String::new();
        stdin()
            .read_line(&mut text)
            .expect("Something went wrong while reading stdin.");

        text = text
            .strip_suffix("\n")
            .map(|s| s.to_string())
            .unwrap_or(text);

        if text.to_lowercase() == "y" {
            return true;
        } else if text.to_lowercase() == "n" {
            return false;
        } else {
            print!("Please enter 'y' or 'n'.\nTry again: ");
        }
    }
}

fn read_string() -> String {
    let _ = stdout().flush();
    let mut text = String::new();
    stdin()
        .read_line(&mut text)
        .expect("Something went wrong while reading stdin.");

    text = text
        .strip_suffix("\n")
        .map(|s| s.to_string())
        .unwrap_or(text);

    return text;
}

fn current_dt_stamp() -> String {
    return format_date_time(chrono::Utc::now());
}

fn format_local_into_utc(dt: DateTime<Local>) -> String {
    return format_date_time(DateTime::from(dt));
}

fn format_date_time(dt: DateTime<Utc>) -> String {
    return dt.format("%Y%m%dT%H%M%SZ").to_string();
}

#[cfg(test)]
mod tests {
    use crate::format_date_time;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_format_date_time() {
        let time = Utc.ymd(2021, 2, 17).and_hms(01, 0, 0);

        assert_eq!("20210217T010000", format_date_time(time));
    }
}
