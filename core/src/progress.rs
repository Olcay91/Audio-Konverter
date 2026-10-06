/// Wertet die `-progress pipe:1`-Ausgabe von ffmpeg aus.
pub struct ProgressParser {
    duration_us: Option<f64>,
}

#[derive(Debug, PartialEq)]
pub enum ProgressEvent {
    /// Anteil 0.0–1.0
    Fraction(f64),
    End,
}

impl ProgressParser {
    pub fn new(duration_secs: Option<f64>) -> Self {
        Self {
            duration_us: duration_secs.filter(|d| *d > 0.0).map(|d| d * 1_000_000.0),
        }
    }

    pub fn feed(&self, line: &str) -> Option<ProgressEvent> {
        let (key, value) = line.trim().split_once('=')?;
        match key {
            // out_time_ms enthält trotz des Namens Mikrosekunden (ältere ffmpeg-Versionen).
            "out_time_us" | "out_time_ms" => {
                let us: f64 = value.parse().ok()?;
                let total = self.duration_us?;
                Some(ProgressEvent::Fraction((us / total).clamp(0.0, 1.0)))
            }
            "progress" if value == "end" => Some(ProgressEvent::End),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_fraction() {
        let p = ProgressParser::new(Some(10.0));
        assert_eq!(p.feed("out_time_us=5000000"), Some(ProgressEvent::Fraction(0.5)));
        assert_eq!(p.feed("out_time_us=N/A"), None);
        assert_eq!(p.feed("progress=end"), Some(ProgressEvent::End));
        assert_eq!(p.feed("bitrate=320.0kbits/s"), None);
    }

    #[test]
    fn unknown_duration_gives_no_fraction() {
        assert_eq!(ProgressParser::new(None).feed("out_time_us=1"), None);
    }
}
