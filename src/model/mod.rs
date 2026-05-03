use std::fmt;
use std::str::FromStr;
mod ttml;

static DATA_SECTION_SPLIT_MARKER: &str = "\n\n[//]";
static LINE_BY_LINE_TIMESTAMP_MARKER: &str = "[lbl]";
static LINE_SYLABLE_KEYFRAME_MARKER: &str = "[lsk]";

#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe {
    pub time: f32,
    pub index: f32, // TODO: use progress
}

// represents segments of a lyrics.
// sorted by time. segements with [bracketed sections] are first element of the group.
#[derive(Clone, Debug, PartialEq)]
pub struct TextSegment {
    pub text: String,
    pub keyframes: Vec<Keyframe>,
    pub part: Option<String>,
    pub start: f32,
    pub end: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationData {
    pub lines: Vec<TextSegment>,
}

impl Keyframe {
    pub fn to_string_pct(&self, line_len: f32) -> String {
        let pct = if line_len > 0.0 {
            self.index / line_len
        } else {
            0.0
        };
        format!("{:.3}/{:.3}", self.time, pct)
    }

    // Helper: requires context
    pub fn from_string_pct(s: &str, line_len: f32) -> Option<Keyframe> {
        let (time_str, pct_str) = s.split_once('/')?;
        let time: f32 = time_str.parse().ok()?;
        let pct: f32 = pct_str.parse().ok()?;
        let index = pct * line_len;
        Some(Keyframe { time, index })
    }
}

impl TextSegment {
    pub fn new(text: String, start: f32, end: f32) -> Self {
        Self {
            text,
            start,
            end,
            part: None,
            keyframes: Vec::new(),
        }
    }

    pub fn get_current_index(&self, rel_time: f32) -> f32 {
        if self.keyframes.is_empty() {
            let duration = self.end - self.start;
            let t = rel_time / duration;
            return (self.text.len() as f32) * t;
        }

        for i in 0..self.keyframes.len() - 1 {
            let k1 = &self.keyframes[i];
            let k2 = &self.keyframes[i + 1];
            if rel_time >= k1.time && rel_time <= k2.time {
                let t = (rel_time - k1.time) / (k2.time - k1.time);
                return k1.index + (k2.index - k1.index) * t;
            }
        }
        self.keyframes.last().map(|k| k.index).unwrap_or(0.0)
    }

    pub fn sort_keyframes(&mut self) {
        self.keyframes.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn add_keyframe(&mut self, time: f32, index: f32) -> &mut Self {
        self.keyframes.push(Keyframe { time, index });
        self.sort_keyframes();
        self
    }

    pub fn add_kf_pct(&mut self, time: f32, pct: f32) -> &mut Self {
        let index = (self.text.len() as f32 * pct).floor();
        self.add_keyframe(time, index)
    }

    pub fn to_string(&self) -> String {
        // if part exists: prefix with [part]\n + segment.text
        if let Some(part) = &self.part {
            return format!("[{}]\n{}\n", part, self.text);
        }
        self.text.clone()
    }

    fn is_boundary(&self, kf_idx: usize) -> bool {
        kf_idx == 0 || kf_idx == self.keyframes.len() - 1
    }
}

impl AnimationData {
    pub fn new() -> Self {
        let empty_line = TextSegment::new(" ".to_string(), 0.0, 0.0);

        Self { lines: vec![empty_line]}
    }

    pub fn compile(&self) -> String {
        self.to_string()
    }

    pub fn add_line(&mut self, text: &str, start: f32, end: f32) -> &mut TextSegment {
        let line = TextSegment::new(text.to_string(), start, end);
        self.lines.push(line);
        self.lines.last_mut().unwrap()
    }

    pub fn gap(&mut self, index: usize) {
        let previous = if index == 0 {
            0.0
        } else {
            match self.lines.get(index - 1) {
                Some(line) => line.end,
                None => 0.0,
            }
        };
        let next = match self.lines.get(index) {
            Some(line) => line.start,
            None => previous + 0.1,
        };

        let value = TextSegment {
            text: "".to_string(),
            keyframes: Vec::new(),
            part: None,
            start: previous,
            end: next,
        };

        if self.lines.len() < index {
            self.lines.push(value)
        } else {
            self.lines.insert(index, value)
        }
    }

    pub fn _sort_lines(&mut self) {
        self.lines.sort_by(|a, b| {
            a.start
                .partial_cmp(&b.start)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn is_boundary(&self, line_idx: usize, kf_idx: usize) -> bool {
        self.lines[line_idx].is_boundary(kf_idx)
    }

    pub fn add_keyframe(&mut self, line_idx: usize, time: f32, position: f32) {
        self.lines[line_idx].keyframes.push(Keyframe {
            time,
            index: position,
        });
        self.lines[line_idx].sort_keyframes();
    }

    pub fn edit_keyframe_time(&mut self, line_idx: usize, kf_idx: usize, delta: f32) {
        let line = &mut self.lines[line_idx];
        let rel_time = (line.keyframes[kf_idx].time + 0.05 * delta).max(0.0);
        line.keyframes[kf_idx].time = rel_time;
    }

    pub fn edit_keyframe_position(&mut self, line_idx: usize, kf_idx: usize, delta: f32) {
        if self.is_boundary(line_idx, kf_idx) {
            return;
        }
        self.lines[line_idx].keyframes[kf_idx].index += delta;
        self.lines[line_idx].sort_keyframes();
    }

    pub fn delete_keyframe(&mut self, line_idx: usize, kf_idx: usize) {
        if self.is_boundary(line_idx, kf_idx) {
            return;
        }
        self.lines[line_idx].keyframes.remove(kf_idx);
    }

    pub fn add_trailing_empty(&mut self) {
        if let Some(last) = self.lines.last() {
            if (last.text.trim() != "") {
                self.add_line(" ", last.end + 0.05, last.end + 5.0);
            }
        }
    }
}

// This allows you to do: my_data.to_string();
impl fmt::Display for AnimationData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut line_strings = Vec::new();
        let mut lines_timestamp = Vec::new();
        let mut lines_keyframes = Vec::new();

        for line in &self.lines {
            // 1. Clean Text
            let sanitized_text: String = line
                .text
                .chars()
                .filter(|&c| !c.is_control() && c != '/' && c != '[' && c != ']')
                .collect();

            if let Some(part) = &line.part {
                line_strings.push(format!("{}\n{}", part, sanitized_text));
            } else {
                line_strings.push(sanitized_text);
            }

            lines_timestamp.push(format!("{:.3}/{:.3}", line.start, line.end));

            let line_len = line.text.len() as f32;
            let kfs: Vec<String> = line
                .keyframes
                .iter()
                .map(|kf| kf.to_string_pct(line_len))
                .collect();
            lines_keyframes.push(format!("({})", kfs.join(",")));
        }

        let s1 = line_strings.join("\n");
        let s2 = lines_timestamp.join(",");
        let s3 = lines_keyframes.join(",");

        write!(
            f,
            "{}\n{}\n{}[{}]\n{}[{}]\n",
            s1,
            DATA_SECTION_SPLIT_MARKER,
            LINE_BY_LINE_TIMESTAMP_MARKER,
            s2,
            LINE_SYLABLE_KEYFRAME_MARKER,
            s3,
        )
    }
}

impl AnimationData {
    pub fn parse_lines(&mut self, text: &str) {
        let mut parts: Option<String> = None;
        for line in text.lines() {
            if line.starts_with("[") && line.ends_with("]") {
                parts = Some(line.to_string());
            } else {
                if let Some(part) = parts {
                    self.add_line(line, 0.0, 0.0).part = Some(part);
                    parts = None;
                } else {
                    self.add_line(line, 0.0, 0.0);
                }
            }
        }
    }

    pub fn parse_timestamps(&mut self, timestamps: &str) {
        let timestamps: Vec<&str> = timestamps.split(',').collect();
        for i in 0..timestamps.len() {
            if i >= self.lines.len() {
                break;
            }
            let timestamp: Vec<&str> = timestamps[i].split('/').collect();
            if timestamp.len() == 2 {
                let start = timestamp[0].trim().parse::<f32>().unwrap_or(0.0);
                let end = timestamp[1].trim().parse::<f32>().unwrap_or(0.0);
                self.lines[i].start = start;
                self.lines[i].end = end;
            }
        }
    }

    pub fn parse_keyframes(&mut self, keyframes: &str) {
        let keyframes: Vec<&str> = keyframes.split(',').collect();
        for i in 0..keyframes.len() {
            if i >= self.lines.len() {
                break;
            }
            let line_len = self.lines[i].text.len() as f32;
            let group = keyframes[i].trim_matches(|c| c == '(' || c == ')');
            if group.is_empty() {
                continue;
            }

            for kf_entry in group.split(',') {
                if let Some(keyframe) = Keyframe::from_string_pct(kf_entry.trim(), line_len) {
                    self.lines[i].keyframes.push(keyframe);
                }
            }
            self.lines[i].sort_keyframes();
        }
    }

    pub fn extract_section(&self, data_section: &str, marker: &str) -> Result<String, String> {
        let start_idx = data_section
            .find(marker)
            .ok_or(format!("Missing {}", marker))?
            + marker.len();
        let open_bracket = data_section[start_idx..].find('[').ok_or("Missing [")? + start_idx;
        let close_bracket =
            data_section[open_bracket..].find(']').ok_or("Missing ]")? + open_bracket;
        Ok(data_section[open_bracket + 1..close_bracket].to_string())
    }
}

impl FromStr for AnimationData {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let _draft = true;
        let mut new_data = AnimationData::new();

        let sections: Vec<&str> = input.split(DATA_SECTION_SPLIT_MARKER).collect();
        let text_section = sections[0].replace("\n\n\n", "\n\n");

        new_data.parse_lines(text_section.as_str());

        if sections.len() < 2 {
            return Err("Format error: Missing [//] separator".to_string());
        }
        let data_section = sections[1].trim();

        let lbl_raw = new_data.extract_section(data_section, LINE_BY_LINE_TIMESTAMP_MARKER)?;
        let lsk_raw = new_data.extract_section(data_section, LINE_SYLABLE_KEYFRAME_MARKER)?;

        new_data.parse_timestamps(&lbl_raw);
        new_data.parse_keyframes(&lsk_raw);
        Ok(new_data)
    }
}

#[test]
fn parse_test() {
    let mut test_data = AnimationData::default();
    test_data.gap(0);
    test_data.add_line("line1", 0.0, 1.0);
    test_data.add_line("line2", 1.0, 2.0);
    test_data.add_line("line3", 2.0, 3.0);
    test_data.gap(5);
    test_data.add_line("line4", 3.0, 4.0);
    test_data.add_line("line5", 4.0, 5.0);
    test_data.gap(7);
    test_data.lines[1].part = Some("[a]".to_string());
    test_data.lines[5].part = Some("[b]".to_string());

    println!("-----");
    let animated_text = test_data.compile();
    println!("{}", animated_text);
    println!("-----");

    let result = animated_text.parse::<AnimationData>();
    match result.clone() {
        Ok(data) => {
            assert_eq!(data.lines[0].text, "".to_string());
            assert_eq!(data.lines.len(), 8);
            assert_eq!(data.lines[1].part, Some("[a]".to_string()));
            assert_eq!(data.lines[2].part, None);
            assert_eq!(data.lines[4].text, "".to_string());
            assert_eq!(data.lines[7].text, "".to_string());
        }
        Err(e) => {
            panic!("{}", e);
        }
    }

    let comp_text = result
        .unwrap()
        .compile()
        .split(DATA_SECTION_SPLIT_MARKER)
        .next()
        .unwrap()
        .to_string();
    let test_text: String = animated_text
        .split(DATA_SECTION_SPLIT_MARKER)
        .next()
        .unwrap()
        .to_string();

    assert_eq!(comp_text, test_text);
}
