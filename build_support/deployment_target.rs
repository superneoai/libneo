#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

pub fn parse(value: &str) -> Result<Version, &'static str> {
    if value.is_empty() {
        return Err("the value is empty");
    }

    let mut components = value.split('.');
    let major = parse_component(components.next())?;
    let minor = components
        .next()
        .map_or(Ok(0), |part| parse_component(Some(part)))?;
    let patch = components
        .next()
        .map_or(Ok(0), |part| parse_component(Some(part)))?;
    if components.next().is_some() {
        return Err("expected at most three numeric components");
    }

    Ok(Version::new(major, minor, patch))
}

fn parse_component(component: Option<&str>) -> Result<u64, &'static str> {
    let component = component.ok_or("a version component is missing")?;
    if component.is_empty() || !component.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("expected dot-separated numeric components");
    }
    component
        .parse()
        .map_err(|_| "a version component is too large")
}

#[cfg(test)]
mod tests {
    use super::{Version, parse};

    #[test]
    fn parses_one_to_three_numeric_components() {
        assert_eq!(parse("26"), Ok(Version::new(26, 0, 0)));
        assert_eq!(parse("26.1"), Ok(Version::new(26, 1, 0)));
        assert_eq!(parse("26.1.2"), Ok(Version::new(26, 1, 2)));
    }

    #[test]
    fn rejects_malformed_versions() {
        for value in ["", ".", "26.", ".1", "26.1.0.1", "26.beta", " 26.1"] {
            assert!(parse(value).is_err(), "{value:?} must be rejected");
        }
    }

    #[test]
    fn compares_versions_component_by_component() {
        let minimum = Version::new(26, 1, 0);
        assert!(parse("26.0.9").unwrap() < minimum);
        assert!(parse("26.1").unwrap() >= minimum);
        assert!(parse("26.2").unwrap() > minimum);
        assert!(parse("27").unwrap() > minimum);
    }
}
