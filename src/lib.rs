use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ProfileData {
    frame: usize,
    frames: Vec<FrameData>,
}
impl ProfileData {
    pub fn new() -> Self {
        Self {
            frame: 0,
            frames: Vec::new(),
        }
    }
    pub fn frame(&mut self) {
        self.frame += 1;
        self.frames.push(FrameData::new());
    }
    pub fn curr_frame_mut(&mut self) -> &mut FrameData {
        self.frames.last_mut().unwrap()
    }
    pub fn stringify(&self) -> String {
        let mut to_return = String::new();
        for (i, frame) in self.frames.iter().enumerate() {
            to_return.push_str(&format!(">-- Frame {}: --<\n{}\n", i, frame.stringify()));
        }
        to_return
    }
}

#[derive(Debug, Clone)]
pub struct FrameData {
    path: Vec<&'static str>,
    start_times: Vec<Instant>,
    segment_times: HashMap<&'static str, Vec<u128>>,
    messages: Vec<(&'static str, String)>,
}
impl FrameData {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            start_times: Vec::new(),
            segment_times: HashMap::new(),
            messages: Vec::new(),
        }
    }
    pub fn enter(&mut self, name: &'static str) {
        self.path.push(name);
        self.start_times.push(Instant::now());
    }
    pub fn leave(&mut self) {
        let time_elapsed = self.start_times.pop().unwrap().elapsed().as_nanos();
        let segment_left = self.path.pop().unwrap();

        if let Some(to_modify) = self.segment_times.get_mut(segment_left) {
            to_modify.push(time_elapsed);
        } else {
            self.segment_times.insert(segment_left, vec![time_elapsed]);
        }
    }
    pub fn post_message(&mut self, input: String) {
        self.messages.push((self.path.last().unwrap(), input));
    }
    fn divide_u128(input: u128, denom: u128) -> f64 {
        let mut to_return = (input / denom) as f64;
        to_return += (input % denom) as f64 / denom as f64;
        to_return
    }
    fn format_time(time: u128) -> String {
        if time >= 10u128.pow(9) {
            format!("{:.2} s", Self::divide_u128(time, 10u128.pow(9)))
        } else if time >= 10u128.pow(6) {
            format!("{:.2} ms", Self::divide_u128(time, 10u128.pow(6)))
        } else if time >= 10u128.pow(3) {
            format!("{:.2} us", Self::divide_u128(time, 10u128.pow(3)))
        } else {
            format!("{} ns", time)
        }
    }
    pub fn stringify(&self) -> String {
        let mut to_return = String::new();
        // Name, average time, mean absolute deviation, min time, max time
        let mut entries: Vec<(&'static str, u32, u128, u128, u128, u128)> = Vec::new();
        for key in self.segment_times.keys() {
            let mut num_calls: u32 = 0;
            let mut average: u128 = 0;
            let mut mad: u128 = 0;
            let mut min: Option<u128> = None;
            let mut max: Option<u128> = None;
            for (i, value) in self.segment_times.get(key).unwrap().iter().enumerate() {
                if *value > average {
                    average += (value - average) / (i as u128 + 1);
                } else {
                    average -= (average - value) / (i as u128 + 1);
                }
                if min.is_none() || *value < min.unwrap() {
                    min = Some(*value);
                }
                if max.is_none() || *value > max.unwrap() {
                    max = Some(*value);
                }
                num_calls += 1;
            }

            for value in self.segment_times.get(key).unwrap().iter() {
                if *value > average {
                    mad += value - average;
                } else {
                    mad += average - value;
                }
            }
            mad /= self.segment_times.get(key).unwrap().iter().count() as u128;

            entries.push((key, num_calls, num_calls as u128 * average, mad, min.unwrap(), max.unwrap()));
        }

        entries.sort_by(|a, b| b.2.cmp(&a.2));

        for entry in entries {
            to_return.push_str(&format!(
                "    * {}: {} calls, mean: {} +- {}, range: {}--{}\n",
                entry.0,
                entry.1,
                Self::format_time(entry.2),
                Self::format_time(entry.3),
                Self::format_time(entry.4),
                Self::format_time(entry.5)
            ));
        }
        to_return
    }
}

pub struct ScopeTimer {}
impl ScopeTimer {
    pub fn new() -> Self {
        Self {}
    }
}
impl Drop for ScopeTimer {
    fn drop(&mut self) {
        profiler_leave!();
    }
}

pub static PROFILE_RECORD: Lazy<RwLock<ProfileData>> =
    Lazy::new(|| RwLock::new(ProfileData::new()));

#[cfg(feature = "runprof")]
#[macro_export]
macro_rules! profiler_enter {
    ($annotation:literal) => {
        PROFILE_RECORD
            .write()
            .unwrap()
            .curr_frame_mut()
            .enter(concat!("[", file!(), ":", line!(), "] (", $annotation, ")"))
    };
}

#[cfg(feature = "runprof")]
#[macro_export]
macro_rules! profiler_leave {
    () => {
        PROFILE_RECORD.write().unwrap().curr_frame_mut().leave()
    };
    ($annotation:literal) => {
        PROFILE_RECORD.write().unwrap().curr_frame_mut().leave()
    };
}

#[cfg(feature = "runprof")]
#[macro_export]
macro_rules! profiler_message {
    ($annotation:expr) => {
        PROFILE_RECORD
            .write()
            .unwrap()
            .curr_frame_mut()
            .post_message($annotation)
    };
}

#[cfg(feature = "runprof")]
#[macro_export]
macro_rules! profiler_frame {
    () => {
        PROFILE_RECORD.write().unwrap().frame()
    };
}

#[cfg(feature = "runprof")]
#[macro_export]
macro_rules! profiler_data {
    () => {
        PROFILE_RECORD.read().unwrap().stringify()
    };
}

#[cfg(feature = "runprof")]
#[macro_export]
macro_rules! profile_scope {
    ($annotation:literal) => {
        PROFILE_RECORD
            .write()
            .unwrap()
            .curr_frame_mut()
            .enter(concat!("[", file!(), ":", line!(), "] (", $annotation, ")"));
        let _scope_marker = ScopeTimer::new();
    };
}

#[cfg(not(feature = "runprof"))]
#[macro_export]
macro_rules! profiler_enter {
    ($annotation:literal) => {};
}

#[cfg(not(feature = "runprof"))]
#[macro_export]
macro_rules! profiler_leave {
    () => {};
    ($annotation:literal) => {};
}

#[cfg(not(feature = "runprof"))]
#[macro_export]
macro_rules! profiler_message {
    ($annotation:expr) => {};
}

#[cfg(not(feature = "runprof"))]
#[macro_export]
macro_rules! profiler_frame {
    () => {};
}

#[cfg(not(feature = "runprof"))]
#[macro_export]
macro_rules! profiler_data {
    () => {
        ""
    };
}

#[cfg(not(feature = "runprof"))]
#[macro_export]
macro_rules! profile_scope {
    ($annotation:literal) => {};
}
