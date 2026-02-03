use std::fmt;
use std::str::FromStr;

static DATA_SECTION_SPLIT_MARKER: &str = "[//]";
static LINE_BY_LINE_TIMESTAMP_MARKER: &str = "[lbl]";
static LINE_SYLABLE_KEYFRAME_MARKER: &str = "[lsk]";

#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe {
    pub time: f32,
    pub index: f32, // TODO: use progress
}

#[derive(Clone, Debug, PartialEq)]
pub struct LyricLine {
    pub text: String,
    pub part: Option<String>,
    pub start: f32,
    pub end: f32,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationData {
    // TODO: Implement as a flat map
    pub lines: Vec<LyricLine>,
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

impl LyricLine {
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
            return 0.0;
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
}

impl AnimationData {
    pub fn demo() -> Self {
        let mut data = AnimationData::default();

        data.add_line("City of stars", 0.0, 3.42)
            .add_kf_pct(0.0, 0.0)
            .add_kf_pct(1.2, 0.7)
            .add_kf_pct(3.42, 1.0);
        data.add_line(
            "You never shined so brightly",
            3.42 + 0.5,
            3.42 + 0.5 + 7.112,
        )
        .add_kf_pct(0.0, 0.0)
        .add_kf_pct(0.4, 0.20)
        .add_kf_pct(5.4, 0.90)
        .add_kf_pct(7.0, 1.0);
        data
    }

    pub fn add_line(&mut self, text: &str, start: f32, end: f32) -> &mut LyricLine {
        let line = LyricLine::new(text.to_string(), start, end);
        self.lines.push(line);
        self.lines.last_mut().unwrap()
    }
}

// --- STANDARD TRAIT IMPL (toString / fromString) ---

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
                line_strings.push(format!("\n[{}]\n{}", part, sanitized_text));
            } else {
                line_strings.push(sanitized_text);
            }

            // 2. Timestamps
            lines_timestamp.push(format!("{:.3}/{:.3}", line.start, line.end));

            // 3. Keyframes
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
            "{}\n{}\n{}[{}]\n{}[{}]",
            s1,
            DATA_SECTION_SPLIT_MARKER,
            LINE_BY_LINE_TIMESTAMP_MARKER,
            s2,
            LINE_SYLABLE_KEYFRAME_MARKER,
            s3,
        )
    }
}

// This allows you to do: "some_string".parse::<AnimationData>();
impl FromStr for AnimationData {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let sections: Vec<&str> = input.split(DATA_SECTION_SPLIT_MARKER).collect();
        if sections.len() < 2 {
            return Err("Format error: Missing [//] separator".to_string());
        }

        let text_section = sections[0].trim();
        let data_section = sections[1].trim();

        // 1. Parse Lines
        let mut lines = Vec::new();
        let mut current_part: Option<String> = None;

        for line_raw in text_section.lines() {
            let trimmed = line_raw.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_part = Some(trimmed[1..trimmed.len() - 1].to_string());
            } else {
                let mut line = LyricLine::new(trimmed.to_string(), 0.0, 0.0);
                line.part = current_part.clone();
                lines.push(line);
            }
        }

        // 2. Extract Helpers
        let extract_data = |marker: &str| -> Result<String, String> {
            let start_idx = data_section
                .find(marker)
                .ok_or(format!("Missing {}", marker))?
                + marker.len();
            let open_bracket = data_section[start_idx..].find('[').ok_or("Missing [")? + start_idx;
            let close_bracket =
                data_section[open_bracket..].find(']').ok_or("Missing ]")? + open_bracket;
            Ok(data_section[open_bracket + 1..close_bracket].to_string())
        };

        // 3. Parse Metadata
        let lbl_raw = extract_data(LINE_BY_LINE_TIMESTAMP_MARKER)?;
        let lsk_raw = extract_data(LINE_SYLABLE_KEYFRAME_MARKER)?;

        let ts_pairs: Vec<&str> = lbl_raw.split(',').collect();
        if ts_pairs.len() != lines.len() {
            return Err("Line count mismatch with timestamps".to_string());
        }

        for (i, pair) in ts_pairs.iter().enumerate() {
            let parts: Vec<&str> = pair.split('/').collect();
            if parts.len() == 2 {
                lines[i].start = parts[0].parse().unwrap_or(0.0);
                lines[i].end = parts[1].parse().unwrap_or(0.0);
            }
        }

        // 4. Parse Keyframes
        let kf_groups: Vec<&str> = lsk_raw.split("),(").collect();
        for (i, group) in kf_groups.iter().enumerate() {
            if i >= lines.len() {
                break;
            }
            let clean_group = group.trim_matches(|c| c == '(' || c == ')');
            let line_len = lines[i].text.len() as f32;

            for kf_entry in clean_group.split(',') {
                if let Some(keyframe) = Keyframe::from_string_pct(kf_entry, line_len) {
                    lines[i].keyframes.push(keyframe);
                }
            }
            lines[i].sort_keyframes();
        }

        Ok(AnimationData { lines })
    }
}
