use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, TimeZone};
use itertools::join;
use std::error::Error;
use std::process::Command;
use task_hookrs::task::Task;
use task_hookrs::uda::UDAValue;

pub fn vague_format_date_time(from_dt: NaiveDateTime, to_dt: NaiveDateTime) -> String {
    let mut seconds = (to_dt - from_dt).num_seconds();
    let minus: &str;

    if seconds < 0 {
        seconds *= -1;
        minus = "-";
    } else {
        minus = "";
    }

    if seconds >= 60 * 60 * 24 * 365 {
        return format!("{}{}y", minus, seconds / 86400 / 365);
    } else if seconds >= 60 * 60 * 24 * 90 {
        return format!("{}{}mo", minus, seconds / 60 / 60 / 24 / 30);
    } else if seconds >= 60 * 60 * 24 * 14 {
        return format!("{}{}w", minus, seconds / 60 / 60 / 24 / 7);
    } else if seconds >= 60 * 60 * 24 {
        return format!("{}{}d", minus, seconds / 60 / 60 / 24);
    } else if seconds >= 60 * 60 {
        return format!("{}{}h", minus, seconds / 60 / 60);
    } else if seconds >= 60 {
        return format!("{}{}min", minus, seconds / 60);
    } else {
        return format!("{}{}s", minus, seconds);
    }
}

pub struct TaskReportTable {
    pub labels: Vec<String>,
    pub columns: Vec<String>,
    pub tasks: Vec<Vec<String>>,
    pub virtual_tags: Vec<String>,
    pub description_width: usize,
}

impl TaskReportTable {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let virtual_tags = vec![
            "PROJECT",
            "BLOCKED",
            "UNBLOCKED",
            "BLOCKING",
            "DUE",
            "DUETODAY",
            "TODAY",
            "OVERDUE",
            "WEEK",
            "MONTH",
            "QUARTER",
            "YEAR",
            "ACTIVE",
            "SCHEDULED",
            "PARENT",
            "CHILD",
            "UNTIL",
            "WAITING",
            "ANNOTATED",
            "READY",
            "YESTERDAY",
            "TOMORROW",
            "TAGGED",
            "PENDING",
            "COMPLETED",
            "DELETED",
            "UDA",
            "ORPHAN",
            "PRIORITY",
            "PROJECT",
            "LATEST",
            "RECURRING",
            "INSTANCE",
        ];
        let mut task_report_table = Self {
            labels: vec![],
            columns: vec![],
            tasks: vec![vec![]],
            virtual_tags: virtual_tags.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            description_width: 100,
        };
        task_report_table.export_headers()?;
        Ok(task_report_table)
    }

    pub fn export_headers(&mut self) -> Result<(), Box<dyn Error>> {
        self.columns = vec![];
        self.labels = vec![];

        let output = Command::new("task").arg("show").arg("report.next.columns").output()?;
        let data = String::from_utf8_lossy(&output.stdout);

        for line in data.split('\n') {
            if line.starts_with("report.next.columns") {
                let column_names = line.split(' ').collect::<Vec<_>>()[1];
                for column in column_names.split(',') {
                    self.columns.push(column.to_string());
                }
            }
        }

        let output = Command::new("task").arg("show").arg("report.next.labels").output()?;
        let data = String::from_utf8_lossy(&output.stdout);

        for line in data.split('\n') {
            if line.starts_with("report.next.labels") {
                let label_names = line.split(' ').collect::<Vec<_>>()[1];
                for label in label_names.split(',') {
                    self.labels.push(label.to_string());
                }
            }
        }
        Ok(())
    }

    pub fn generate_table(&mut self, tasks: &[Task]) {
        self.tasks = vec![];

        // get all tasks as their string representation
        for task in tasks {
            if self.columns.is_empty() {
                break;
            }
            let mut item = vec![];
            for name in &self.columns {
                let s = self.get_string_attribute(name, &task, tasks);
                item.push(s);
            }
            self.tasks.push(item)
        }
    }

    pub fn simplify_table(&mut self) -> (Vec<Vec<String>>, Vec<String>) {
        // find which columns are empty
        let null_columns_len;
        if !self.tasks.is_empty() {
            null_columns_len = self.tasks[0].len();
        } else {
            return (vec![], vec![]);
        }

        let mut null_columns = vec![0; null_columns_len];
        for task in &self.tasks {
            for (i, s) in task.iter().enumerate() {
                null_columns[i] += s.len();
            }
        }

        // filter out columns where everything is empty
        let mut tasks = vec![];
        for task in &self.tasks {
            let t = task.clone();
            let t: Vec<String> = t
                .iter()
                .enumerate()
                .filter(|&(i, _)| null_columns[i] != 0)
                .map(|(_, e)| e.to_owned())
                .collect();
            tasks.push(t);
        }

        // filter out header where all columns are empty
        let headers: Vec<String> = self
            .labels
            .iter()
            .enumerate()
            .filter(|&(i, _)| null_columns[i] != 0)
            .map(|(_, e)| e.to_owned())
            .collect();

        (tasks, headers)
    }

    pub fn get_string_attribute(&self, attribute: &str, task: &Task, tasks: &[Task]) -> String {
        match attribute {
            "id" => task.id().unwrap_or_default().to_string(),
            "due.relative" => match task.due() {
                Some(v) => vague_format_date_time(Local::now().naive_utc(), NaiveDateTime::new(v.date(), v.time())),
                None => "".to_string(),
            },
            "until.remaining" => match task.until() {
                Some(v) => vague_format_date_time(Local::now().naive_utc(), NaiveDateTime::new(v.date(), v.time())),
                None => "".to_string(),
            },
            "entry.age" => vague_format_date_time(
                NaiveDateTime::new(task.entry().date(), task.entry().time()),
                Local::now().naive_utc(),
            ),
            "start.age" => match task.start() {
                Some(v) => vague_format_date_time(NaiveDateTime::new(v.date(), v.time()), Local::now().naive_utc()),
                None => "".to_string(),
            },
            "status.short" => task.status().to_string().chars().next().unwrap().to_string(),
            "status" => task.status().to_string(),
            "priority" => match task.priority() {
                Some(p) => p.to_owned(),
                None => "".to_string(),
            },
            "project" => match task.project() {
                Some(p) => p.to_string(),
                None => "".to_string(),
            },
            "depends.count" => match task.depends() {
                Some(v) => {
                    if v.is_empty() {
                        "".to_string()
                    } else {
                        format!("{}", v.len())
                    }
                }
                None => "".to_string(),
            },
            "depends" => match task.depends() {
                Some(v) => {
                    if v.is_empty() {
                        "".to_string()
                    } else {
                        let mut dt = vec![];
                        for u in v {
                            if let Some(t) = tasks.iter().find(|t| t.uuid() == u) {
                                dt.push(t.id().unwrap())
                            }
                        }
                        join(dt.iter().map(|i| i.to_string()), " ")
                    }
                }
                None => "".to_string(),
            },
            "tags.count" => match task.tags() {
                Some(v) => {
                    let t = v.iter().filter(|t| !self.virtual_tags.contains(t)).cloned().count();
                    if t == 0 {
                        "".to_string()
                    } else {
                        t.to_string()
                    }
                }
                None => "".to_string(),
            },
            "tags" => match task.tags() {
                Some(v) => v
                    .iter()
                    .filter(|t| !self.virtual_tags.contains(t))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(","),
                None => "".to_string(),
            },
            "description.count" => {
                let c = match task.annotations() {
                    Some(a) => format!("[{}]", a.len()),
                    None => format!(""),
                };
                format!("{} {}", task.description().to_string(), c)
            }
            "description.truncated_count" => {
                let c = match task.annotations() {
                    Some(a) => format!(" [{}]", a.len()),
                    None => format!(""),
                };
                let mut d = task.description().to_string();
                let mut end = self.description_width;
                if self.description_width >= c.len() {
                    end = self.description_width - c.len();
                }
                while end < d.len() && !d.is_char_boundary(end) {
                    end += 1;
                }
                d.truncate(end);
                if d != *task.description() {
                    d = format!("{}…", d);
                }
                format!("{}{}", d, c)
            }
            "description.truncated" => {
                let mut d = task.description().to_string();
                let mut end = self.description_width;
                while end < d.len() && !d.is_char_boundary(end) {
                    end += 1;
                }
                d.truncate(end);
                if d != *task.description() {
                    d = format!("{}…", d);
                }
                d
            }
            "description" => task.description().to_string(),
            "urgency" => match &task.urgency() {
                Some(f) => format!("{:.2}", *f),
                None => "0.00".to_string(),
            },
            s => {
                let u = &task.uda();
                let v = u.get(s);
                if v.is_none() {
                    return "".to_string();
                }
                match v.unwrap() {
                    UDAValue::Str(s) => s.to_string(),
                    UDAValue::F64(f) => f.to_string(),
                    UDAValue::U64(u) => u.to_string(),
                }
            }
        }
    }
}
