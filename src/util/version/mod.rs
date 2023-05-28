use std::cmp::Ordering;

use anyhow::bail;

use crate::hard_error;

#[derive(Clone, Debug)]
pub struct VersionMatcher {
    pub(crate) range: Vec<VersionMatchRange>,
    pub(crate) antirange: Vec<VersionMatchRange>,
}

#[derive(Clone, Debug)]
pub struct VersionMatchRange {
    pub(crate) start_vp: VersionPart,
    pub(crate) end_vp: Option<VersionPart>,
}

#[derive(Clone, Debug)]
pub struct VersionPart {
    pub(crate) points: Vec<(u32,bool)>,
    // no qualifier means newer
    pub(crate) qualifier: Option<Vec<u8>>,
}

impl VersionMatcher {
    pub fn is_empty_recursive(&self) -> bool {
        self.range.iter().any(|r| r.is_empty_recursive() )
    }

    pub fn matches_version(&self, v: &VersionPart) -> bool {
        let mut m = false;

        for l in &self.range {
            m |= l.matches_version(v);
        }

        for l in &self.antirange {
            m &= !l.matches_version(v);
        }

        m
    }
}

impl VersionMatchRange {
    pub fn matches_version(&self, v: &VersionPart) -> bool {
        if self.is_empty_recursive() || v.points.is_empty() {
            return false;
        }

        if let Some(end_vp) = &self.end_vp {
            self.in_range(end_vp, v)
        } else {
            self.in_startvp(v)
        }
    }

    pub fn is_empty_recursive(&self) -> bool {
        if self.start_vp.points.is_empty() {
            return true;
        }
        if let Some(end_vp) = &self.end_vp {
            if end_vp.points.is_empty() {
                return true;
            }
        }
        false
    }

    fn in_startvp(&self, v: &VersionPart) -> bool {
        let v_points = self.start_vp.points.len().max(v.points.len());

        for i in 0..v_points {
            let &(left_num,left_plus) = self.start_vp.points.get(i).unwrap_or(&(0,true));
            let &(right_num,_) = v.points.get(i).unwrap_or(&(0,false));

            if left_plus && right_num > left_num {
                return true;
            }

            if right_num != left_num {
                return false;
            }
        }

        let left_q = self.start_vp.qualifier.as_deref().unwrap_or(&[0xFF]);
        let right_q = v.qualifier.as_deref().unwrap_or(&[0xFF]);

        if right_q < left_q {
            return false;
        }

        true
    }

    fn in_range(&self, end_vp: &VersionPart, v: &VersionPart) -> bool {
        fn cmp_vp(left: &[(u32,bool)], right: &[(u32,bool)], left_default: u32, right_default: u32) -> Ordering {
            for i in 0..left.len().max(right.len()) {
                let l = left.get(i).map(|&(v,_)| v).unwrap_or(left_default);
                let r = right.get(i).map(|&(v,_)| v).unwrap_or(right_default);

                match l.cmp(&r) {
                    Ordering::Equal => {},
                    v @ _ => return v,
                }
            }

            Ordering::Equal
        }

        if 
            cmp_vp(&v.points, &self.start_vp.points, 0, 0) == Ordering::Less ||
            cmp_vp(&v.points, &end_vp.points, 0, u32::MAX) == Ordering::Greater
        { return false; }

        let left_min_q = self.start_vp.qualifier.as_deref().unwrap_or(&[0xFF]);
        let left_max_q = end_vp.qualifier.as_deref().unwrap_or(&[0xFF]);
        let right_q = v.qualifier.as_deref().unwrap_or(&[0xFF]);

        if left_max_q < left_min_q {
            hard_error!("Swapped conf game version range");
        }

        if right_q < left_min_q || right_q > left_max_q {
            return false;
        }

        true
    }
}

impl VersionMatcher {
    pub fn parse(v: &str) -> anyhow::Result<Self> {
        let mut range = Vec::new();
        let mut antirange = Vec::new();

        for v in v.split(',') {
            let v = v.trim_start();
            if v.starts_with('!') {
                antirange.push(VersionMatchRange::parse_str(&v[1..])?);
            } else {
                range.push(VersionMatchRange::parse_str(v)?);
            }
        }

        Ok(dbg!(Self {
            range,
            antirange,
        }))
    }
}

impl VersionMatchRange {
    pub fn parse_str(v: &str) -> anyhow::Result<Self> {
        let mut cursor = 0;
        let input = v.as_bytes();
        Self::parse(input, &mut cursor)
    }

    pub fn parse(input: &[u8], cursor: &mut usize) -> anyhow::Result<Self> {
        let end_chars = b",";
        let illegal_chars = b"([]){}";
        let sep_char = b"-";
        let skip_chars = b" \r\n";

        let start_vp = VersionPart::parse(input, cursor)?;
        let mut end_vp = None;

        if *cursor < input.len() && illegal_chars.contains(&input[*cursor]) {
            bail!("Illegal parser char");
        }

        while *cursor < input.len() && skip_chars.contains(&input[*cursor]) {
            *cursor += 1;
        }
        if *cursor < input.len() && sep_char.contains(&input[*cursor]) {
            *cursor += 1;
            end_vp = Some(VersionPart::parse(input, cursor)?);
            while *cursor < input.len() && skip_chars.contains(&input[*cursor]) {
                *cursor += 1;
            }
        }

        if *cursor < input.len() && !end_chars.contains(&input[*cursor]) {
            bail!("Version range must end with comma and can't have more than one minus");
        }

        Ok(Self {
            start_vp: start_vp,
            end_vp,
        })
    }
}

impl VersionPart {
    pub fn empty() -> Self {
        Self {
            points: vec![],
            qualifier: None,
        }
    }

    pub fn parse_str(v: &str) -> anyhow::Result<Self> {
        let mut cursor = 0;
        let input = v.as_bytes();
        Self::parse(input, &mut cursor)
    }
    
    pub fn parse(input: &[u8], cursor: &mut usize) -> anyhow::Result<Self> {
        let end_chars = b"-,";
        let illegal_chars = b"([]){}";
        let dot = b".w";
        let number_end = b"_";
        let number_plus = b"+";
        let numbers = b"0123456789";
        let skip_chars = b" \r\n";

        let mut build_number = Vec::with_capacity(4);
        let mut bn_plus = false;
        let mut points = Vec::<(u32,bool)>::with_capacity(4);
        let mut qualifier = None;
        let mut scan_quali = false;

        loop {
            if *cursor >= input.len() {
                break;
            }

            if skip_chars.contains(&input[*cursor]) {
                *cursor += 1;
                continue;
            }

            if illegal_chars.contains(&input[*cursor]) {
                bail!("Illegal parser char");
            }

            if end_chars.contains(&input[*cursor]) {
                break;
            }

            if scan_quali {
                qualifier.get_or_insert_with(|| Vec::new() ).push(input[*cursor]);
                *cursor += 1;
                continue;
            }

            if numbers.contains(&input[*cursor]) {
                build_number.push(input[*cursor]);
                *cursor += 1;
                continue;
            }

            if number_plus.contains(&input[*cursor]) {
                if build_number.is_empty() {
                    if let Some(last) = points.last_mut() {
                        last.1 = true;
                    }
                } else {
                    bn_plus = true;
                }
                *cursor += 1;
                continue;
            }

            if dot.contains(&input[*cursor]) {
                if build_number.is_empty() {
                    
                } else {
                    let s = std::str::from_utf8(&build_number)?;
                    let num = s.parse()?;
                    points.push((num,bn_plus));
                    build_number.clear();
                }
                
                //bn_plus = false;

                *cursor += 1;
                continue;
            }

            // scan_quali

            scan_quali = true;

            if number_end.contains(&input[*cursor]) {
                *cursor += 1;
            }
        }

        if !build_number.is_empty() {
            let s = std::str::from_utf8(&build_number)?;
            let num = s.parse()?;
            points.push((num,bn_plus));
        }

        Ok(Self {
            points,
            qualifier,
        })
    }
}
